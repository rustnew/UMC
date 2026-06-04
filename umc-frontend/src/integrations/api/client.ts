// UMC REST API client — replaces Supabase

const BASE_URL = import.meta.env.VITE_API_URL ?? "http://localhost:8080";

// ── Token storage ─────────────────────────────────────────────────────────────

const TOKEN_KEY = "umc_access_token";
const REFRESH_KEY = "umc_refresh_token";

export function getAccessToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function getRefreshToken(): string | null {
  return localStorage.getItem(REFRESH_KEY);
}

export function setTokens(access: string, refresh: string): void {
  localStorage.setItem(TOKEN_KEY, access);
  localStorage.setItem(REFRESH_KEY, refresh);
}

export function clearTokens(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_KEY);
}

// ── HTTP helper ───────────────────────────────────────────────────────────────

async function request<T>(
  path: string,
  options: RequestInit = {},
  retry = true
): Promise<T> {
  const token = getAccessToken();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string>),
  };
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const res = await fetch(`${BASE_URL}${path}`, { ...options, headers });

  // Auto-refresh on 401
  if (res.status === 401 && retry) {
    const refreshed = await tryRefresh();
    if (refreshed) return request<T>(path, options, false);
    clearTokens();
    throw new ApiError(401, "Session expired. Please log in again.");
  }

  if (!res.ok) {
    let message = res.statusText;
    try {
      const body = await res.json();
      message = body?.error?.message ?? body?.message ?? message;
    } catch {}
    throw new ApiError(res.status, message);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

async function tryRefresh(): Promise<boolean> {
  const rt = getRefreshToken();
  if (!rt) return false;
  try {
    const data = await fetch(`${BASE_URL}/auth/refresh`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: rt }),
    }).then((r) => (r.ok ? r.json() : null));
    if (!data) return false;
    setTokens(data.access_token, data.refresh_token);
    return true;
  } catch {
    return false;
  }
}

export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = "ApiError";
  }
}

// ── Auth ──────────────────────────────────────────────────────────────────────

export interface UserPublic {
  id: string;
  email: string;
  display_name: string | null;
  plan: string;
  created_at: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: UserPublic;
}

export const auth = {
  register: (email: string, password: string, displayName?: string) =>
    request<AuthResponse>("/auth/register", {
      method: "POST",
      body: JSON.stringify({ email, password, display_name: displayName }),
    }),

  login: (email: string, password: string) =>
    request<AuthResponse>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    }),

  logout: (refreshToken?: string) =>
    request<void>("/auth/logout", {
      method: "POST",
      body: JSON.stringify({ refresh_token: refreshToken }),
    }),

  me: () => request<UserPublic>("/auth/me"),
};

// ── Formats ───────────────────────────────────────────────────────────────────

export interface FormatInfo {
  slug: string;
  name: string;
  can_read: boolean;
  can_write: boolean;
  native: boolean;
  extensions: string[];
  description: string;
}

export interface GraphEdge { from: string; to: string; cost: number; }

export const formats = {
  list: () =>
    request<{ formats: FormatInfo[] }>("/v1/formats").then((r) => r.formats),

  graph: () =>
    request<{ edges: GraphEdge[] }>("/v1/formats/graph").then((r) => r.edges),
};

// ── Upload ────────────────────────────────────────────────────────────────────

export interface UploadResponse {
  upload_id: string;
  filename: string;
  size: number;
  hash: string;
  detected_format: string | null;
}

export async function uploadFile(file: File): Promise<UploadResponse> {
  const token = getAccessToken();
  const formData = new FormData();
  formData.append("file", file);

  const res = await fetch(`${BASE_URL}/v1/upload`, {
    method: "POST",
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: formData,
  });

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new ApiError(res.status, body?.error?.message ?? res.statusText);
  }
  return res.json();
}

// ── Jobs ──────────────────────────────────────────────────────────────────────

export interface ConversionJob {
  id: string;
  user_id: string;
  source_format: string;
  target_format: string;
  validate_mode: string;
  generate_cert: boolean;
  status: "queued" | "running" | "done" | "failed" | "cancelled";
  progress: number;
  tensors_done: number;
  tensors_total: number | null;
  last_tensor: string | null;
  error_message: string | null;
  warnings: string[] | null;
  source_file_size: number | null;
  output_file_size: number | null;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface CreateJobRequest {
  source_format: string;
  target_format: string;
  validate_mode?: string;
  generate_cert?: boolean;
  upload_id: string;
}

export const jobs = {
  create: (req: CreateJobRequest) =>
    request<ConversionJob>("/v1/jobs", {
      method: "POST",
      body: JSON.stringify(req),
    }),

  get: (id: string) => request<ConversionJob>(`/v1/jobs/${id}`),

  list: (params?: { limit?: number; offset?: number; status?: string }) => {
    const q = new URLSearchParams();
    if (params?.limit) q.set("limit", String(params.limit));
    if (params?.offset) q.set("offset", String(params.offset));
    if (params?.status) q.set("status", params.status);
    return request<{ jobs: ConversionJob[] }>(
      `/v1/jobs?${q.toString()}`
    ).then((r) => r.jobs);
  },

  cancel: (id: string) =>
    request<void>(`/v1/jobs/${id}`, { method: "DELETE" }),

  downloadUrl: (id: string) => `${BASE_URL}/v1/jobs/${id}/download`,
};

// ── SSE Progress ──────────────────────────────────────────────────────────────

export interface ProgressEvent {
  job_id: string;
  status: string;
  progress: number;
  tensors_done: number;
  tensors_total: number | null;
  last_tensor: string | null;
  message: string | null;
}

export function subscribeJobProgress(
  jobId: string,
  onEvent: (ev: ProgressEvent) => void,
  onDone?: () => void,
  onError?: (e: Event) => void
): EventSource {
  const token = getAccessToken();
  // SSE: pass token as query param (EventSource doesn't support custom headers)
  const url = `${BASE_URL}/v1/jobs/${jobId}/progress${token ? `?token=${encodeURIComponent(token)}` : ""}`;
  const es = new EventSource(url);

  es.onmessage = (e) => {
    try {
      const data: ProgressEvent = JSON.parse(e.data);
      onEvent(data);
      if (data.status === "done" || data.status === "failed" || data.status === "cancelled") {
        es.close();
        onDone?.();
      }
    } catch {}
  };

  es.onerror = (e) => {
    es.close();
    onError?.(e);
    onDone?.();
  };

  return es;
}

// ── Health ────────────────────────────────────────────────────────────────────

export const health = () =>
  request<{ status: string; version: string }>("/health");
