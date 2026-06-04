import type { CSSProperties } from "react";

/**
 * Stylized brand marks for AI ecosystem companies.
 * Uses a recognizable monogram + brand color rendered as SVG so it scales
 * cleanly across themes. Not the trademarked logos themselves — neutral
 * monograms tinted with each brand's signature color.
 */
export type BrandKey =
  | "pytorch" | "onnx" | "huggingface" | "nvidia" | "apple" | "google"
  | "meta" | "mistral" | "microsoft" | "intel" | "amd" | "qualcomm"
  | "arm" | "tencent" | "alibaba" | "stability" | "anthropic" | "openai"
  | "rockchip" | "baidu" | "xiaomi" | "samsung" | "ibm" | "amazon"
  | "tesla" | "spotify" | "shopify" | "bmw" | "airbus" | "snapchat"
  | "discord" | "github" | "cohere" | "deepseek" | "ggerganov"
  | "apache" | "deepmind" | "huawei" | "berkeley" | "w3c";

export const BRANDS: Record<BrandKey, { name: string; color: string; mono: string }> = {
  pytorch:     { name: "PyTorch",       color: "#EE4C2C", mono: "PT" },
  onnx:        { name: "ONNX",          color: "#005CED", mono: "ON" },
  huggingface: { name: "Hugging Face",  color: "#FFD21E", mono: "🤗" },
  nvidia:      { name: "NVIDIA",        color: "#76B900", mono: "NV" },
  apple:       { name: "Apple",         color: "#A2AAAD", mono: "" },
  google:      { name: "Google",        color: "#4285F4", mono: "G" },
  meta:        { name: "Meta",          color: "#0866FF", mono: "M" },
  mistral:     { name: "Mistral AI",    color: "#FA520F", mono: "Mi" },
  microsoft:   { name: "Microsoft",     color: "#00A4EF", mono: "MS" },
  intel:       { name: "Intel",         color: "#0071C5", mono: "in" },
  amd:         { name: "AMD",           color: "#ED1C24", mono: "▲" },
  qualcomm:    { name: "Qualcomm",      color: "#3253DC", mono: "Q" },
  arm:         { name: "ARM",           color: "#0091BD", mono: "ar" },
  tencent:     { name: "Tencent",       color: "#0052D9", mono: "T" },
  alibaba:     { name: "Alibaba",       color: "#FF6A00", mono: "A" },
  stability:   { name: "Stability AI",  color: "#9B5DE5", mono: "S" },
  anthropic:   { name: "Anthropic",     color: "#D97757", mono: "✦" },
  openai:      { name: "OpenAI",        color: "#10A37F", mono: "○" },
  rockchip:    { name: "Rockchip",      color: "#E60012", mono: "RK" },
  baidu:       { name: "Baidu",         color: "#2932E1", mono: "百" },
  xiaomi:      { name: "Xiaomi",        color: "#FF6700", mono: "MI" },
  samsung:     { name: "Samsung",       color: "#1428A0", mono: "S" },
  ibm:         { name: "IBM",           color: "#0F62FE", mono: "IBM" },
  amazon:      { name: "Amazon",        color: "#FF9900", mono: "a" },
  tesla:       { name: "Tesla",         color: "#E31937", mono: "T" },
  spotify:     { name: "Spotify",       color: "#1DB954", mono: "♪" },
  shopify:     { name: "Shopify",       color: "#95BF47", mono: "S" },
  bmw:         { name: "BMW",           color: "#1C69D4", mono: "B" },
  airbus:      { name: "Airbus",        color: "#00205B", mono: "A" },
  snapchat:    { name: "Snapchat",      color: "#FFFC00", mono: "👻" },
  discord:     { name: "Discord",       color: "#5865F2", mono: "D" },
  github:      { name: "GitHub",        color: "#F0F6FC", mono: "GH" },
  cohere:      { name: "Cohere",        color: "#39594D", mono: "co" },
  deepseek:    { name: "DeepSeek",      color: "#4D6BFE", mono: "🐋" },
  ggerganov:   { name: "G. Gerganov",   color: "#00FF94", mono: "gg" },
  apache:      { name: "Apache",        color: "#D22128", mono: "AP" },
  deepmind:    { name: "DeepMind",      color: "#4285F4", mono: "DM" },
  huawei:      { name: "Huawei",        color: "#E60012", mono: "华" },
  berkeley:    { name: "UC Berkeley",   color: "#003262", mono: "UC" },
  w3c:         { name: "W3C",           color: "#005A9C", mono: "W3" },
};

export function BrandMark({
  brand, size = 28, withName = false, className = "",
}: { brand: BrandKey; size?: number; withName?: boolean; className?: string }) {
  const b = BRANDS[brand];
  const style: CSSProperties = {
    width: size, height: size,
    background: `linear-gradient(135deg, ${b.color}, ${b.color}cc)`,
    color: pickFg(b.color),
  };
  return (
    <span className={`inline-flex items-center gap-2 ${className}`}>
      <span
        style={style}
        className="inline-flex items-center justify-center rounded-md font-mono font-semibold text-[11px] tracking-tight shrink-0 shadow-sm"
        aria-label={b.name}
        title={b.name}
      >
        {b.mono || b.name.slice(0, 2).toUpperCase()}
      </span>
      {withName && <span className="text-sm text-[color:var(--text-2)]">{b.name}</span>}
    </span>
  );
}

function pickFg(hex: string) {
  const c = hex.replace("#", "");
  const r = parseInt(c.slice(0, 2), 16);
  const g = parseInt(c.slice(2, 4), 16);
  const b = parseInt(c.slice(4, 6), 16);
  const yiq = (r * 299 + g * 587 + b * 114) / 1000;
  return yiq >= 160 ? "#0A0A0A" : "#FFFFFF";
}