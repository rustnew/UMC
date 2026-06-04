import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { Download, ShieldCheck, Search, X, CheckCircle2 } from "lucide-react";
import { BrandMark, type BrandKey } from "@/lib/brands";
import { FORMATS } from "@/lib/formats";
import { useEffect, useState } from "react";
import { useTheme } from "@/lib/theme";

export const Route = createFileRoute("/hub")({ component: Page });

type Model = {
  name: string;
  org: BrandKey;
  params: string;
  downloads: string;
  family: string;
  description: string;
  formats: string[];
};

const ALL_TARGETS = ["safetensors","gguf","onnx","coreml","tflite","executorch","mlx","awq","gptq","tensorrt","openvino","mnn","ncnn"];

const MODELS: Model[] = [
  { name: "Llama 3.1 8B Instruct", org: "meta", params: "8.03 B", downloads: "2.4M", family: "LLM", description: "Instruct fine-tune de Meta, idéal pour assistants généralistes locaux.", formats: ALL_TARGETS },
  { name: "Llama 3.1 70B Instruct", org: "meta", params: "70.6 B", downloads: "890k", family: "LLM", description: "Le 70B de référence, raisonnement et code. Inférence GPU multi-cartes.", formats: ["safetensors","gguf","awq","gptq","onnx","tensorrt","openvino","mlx","coreml","executorch"] },
  { name: "Llama 3.1 405B", org: "meta", params: "405 B", downloads: "180k", family: "LLM", description: "Modèle flagship Meta, niveau GPT-4. Quantification massive recommandée.", formats: ["safetensors","gguf","awq","gptq"] },
  { name: "Mistral 7B v0.3", org: "mistral", params: "7.24 B", downloads: "1.8M", family: "LLM", description: "Modèle européen ouvert, excellent ratio qualité/taille.", formats: ALL_TARGETS },
  { name: "Mixtral 8x22B", org: "mistral", params: "141 B", downloads: "320k", family: "MoE", description: "Mixture-of-experts, performance d'un dense 70B au coût d'un 39B actif.", formats: ["safetensors","gguf","awq","onnx","tensorrt"] },
  { name: "Phi-3 Mini 4k Instruct", org: "microsoft", params: "3.82 B", downloads: "1.2M", family: "SLM", description: "Petit, rapide, déployable on-device (mobile, edge, embedded).", formats: ALL_TARGETS },
  { name: "Phi-3 Medium 14B", org: "microsoft", params: "14.0 B", downloads: "410k", family: "SLM", description: "SLM 14B optimisé pour le raisonnement et le code.", formats: ["safetensors","gguf","onnx","tflite","coreml","awq","openvino"] },
  { name: "Gemma 2 9B IT", org: "google", params: "9.24 B", downloads: "740k", family: "LLM", description: "Modèle Google open-weight, base de Gemini Nano.", formats: ["safetensors","gguf","tflite","onnx","mlx","coreml","awq"] },
  { name: "Gemma 2 27B IT", org: "google", params: "27.2 B", downloads: "260k", family: "LLM", description: "Le 27B Google, MMLU 75+, idéal pour fine-tunes verticaux.", formats: ["safetensors","gguf","tflite","onnx","awq","gptq"] },
  { name: "Whisper Large v3", org: "openai", params: "1.55 B", downloads: "920k", family: "ASR", description: "Speech-to-text multilingue de référence, 99 langues.", formats: ["safetensors","onnx","coreml","tflite","openvino","mlx"] },
  { name: "Whisper Medium", org: "openai", params: "769 M", downloads: "540k", family: "ASR", description: "Bon compromis vitesse/qualité pour le edge.", formats: ["safetensors","onnx","coreml","tflite","openvino"] },
  { name: "Stable Diffusion XL 1.0", org: "stability", params: "3.5 B", downloads: "650k", family: "Diffusion", description: "Génération image 1024×1024, base + refiner.", formats: ["safetensors","onnx","coreml","tensorrt","openvino"] },
  { name: "Stable Diffusion 3.5 Large", org: "stability", params: "8.1 B", downloads: "220k", family: "Diffusion", description: "Dernière génération SD, MMDiT architecture.", formats: ["safetensors","onnx","coreml","tensorrt"] },
  { name: "Qwen 2.5 32B Instruct", org: "alibaba", params: "32.5 B", downloads: "410k", family: "LLM", description: "Modèle chinois ouvert, multi-lingue, fort en code.", formats: ["safetensors","gguf","mnn","awq","gptq","onnx"] },
  { name: "Qwen 2.5 Coder 7B", org: "alibaba", params: "7.2 B", downloads: "180k", family: "Code", description: "Spécialisé code, alternative à CodeLlama.", formats: ["safetensors","gguf","mnn","awq","onnx"] },
  { name: "DeepSeek V3", org: "deepseek", params: "671 B", downloads: "180k", family: "MoE", description: "MoE 37B actif, niveau Claude 3.5 Sonnet sur de nombreux benchmarks.", formats: ["safetensors","gguf","awq"] },
  { name: "DeepSeek R1", org: "deepseek", params: "671 B", downloads: "240k", family: "Reasoning", description: "Modèle de raisonnement open-weight, chaînes de pensée natives.", formats: ["safetensors","gguf","awq"] },
  { name: "Stable LM 2 1.6B", org: "stability", params: "1.6 B", downloads: "120k", family: "SLM", description: "Modèle ultra-léger pour le edge.", formats: ALL_TARGETS },
  { name: "Pixtral 12B", org: "mistral", params: "12.0 B", downloads: "95k", family: "VLM", description: "Multimodal vision+texte de Mistral.", formats: ["safetensors","gguf","onnx","awq"] },
  { name: "Llava 1.6 Mistral 7B", org: "mistral", params: "7.0 B", downloads: "210k", family: "VLM", description: "Vision-language, basé Mistral 7B.", formats: ["safetensors","gguf","onnx","coreml"] },
  { name: "FLUX.1 dev", org: "stability", params: "12.0 B", downloads: "380k", family: "Diffusion", description: "Génération image SOTA, qualité photographique.", formats: ["safetensors","onnx","tensorrt"] },
  { name: "CodeLlama 13B Instruct", org: "meta", params: "13.0 B", downloads: "470k", family: "Code", description: "Fine-tune Llama pour la génération de code.", formats: ["safetensors","gguf","onnx","awq","gptq","coreml"] },
  { name: "T5 XXL", org: "google", params: "11.0 B", downloads: "330k", family: "Encoder-Decoder", description: "Text-to-text universel, base de Flan-T5.", formats: ["safetensors","onnx","tflite","openvino"] },
  { name: "BERT Large", org: "google", params: "340 M", downloads: "780k", family: "Encoder", description: "Encodeur historique, encore très utilisé pour search et embeddings.", formats: ALL_TARGETS },
  { name: "RoBERTa Large", org: "meta", params: "355 M", downloads: "510k", family: "Encoder", description: "BERT raffiné par Meta, robuste sur de nombreuses tâches NLP.", formats: ALL_TARGETS },
  { name: "DINOv2 ViT-L", org: "meta", params: "300 M", downloads: "180k", family: "Vision", description: "Embeddings visuels self-supervised, segmentation et classification.", formats: ["safetensors","onnx","coreml","tflite","tensorrt"] },
  { name: "SAM 2 (Segment Anything)", org: "meta", params: "224 M", downloads: "290k", family: "Vision", description: "Segmentation image + vidéo zero-shot.", formats: ["safetensors","onnx","coreml","tensorrt"] },
  { name: "Cohere Command R+", org: "cohere", params: "104 B", downloads: "78k", family: "LLM", description: "Modèle Cohere pour RAG entreprise.", formats: ["safetensors","gguf","awq","onnx"] },
  { name: "Claude Tokenizer", org: "anthropic", params: "—", downloads: "55k", family: "Tokenizer", description: "Tokenizer open-source compatible Claude.", formats: ["safetensors"] },
  { name: "MusicGen Medium", org: "meta", params: "1.5 B", downloads: "130k", family: "Audio", description: "Génération musicale conditionnée par texte.", formats: ["safetensors","onnx","coreml"] },
];

function Page() {
  const { lang } = useTheme();
  const [q, setQ] = useState("");
  const [open, setOpen] = useState<Model | null>(null);
  const filtered = MODELS.filter((m) => m.name.toLowerCase().includes(q.toLowerCase()));

  return (
    <PageStub
      eyebrow="Hub"
      title={lang === "fr" ? "30 modèles pré-convertis, prêts à télécharger." : "30 pre-converted models, ready to download."}
      description={lang === "fr"
        ? "Chaque modèle est disponible dans tous ses formats utiles, certifié, prêt à servir. Cliquez pour voir les détails."
        : "Each model is available in every useful format, certified, ready to serve. Click for details."}
    >
      <div className="relative mb-6 max-w-md">
        <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-[color:var(--text-3)]" />
        <input
          value={q} onChange={(e) => setQ(e.target.value)}
          placeholder={lang === "fr" ? "Rechercher un modèle…" : "Search a model…"}
          className="w-full pl-9 pr-3 py-2.5 rounded-lg bg-[color:var(--bg-2)] border border-[color:var(--border)] text-sm focus:border-[color:var(--green)] outline-none"
        />
      </div>

      <div className="grid md:grid-cols-2 gap-4">
        {filtered.map((m) => (
          <button key={m.name} onClick={() => setOpen(m)}
            className="text-left rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-5 hover:border-[color:var(--text-3)] transition group">
            <div className="flex items-start justify-between gap-4">
              <div className="flex items-start gap-3">
                <BrandMark brand={m.org} size={36} />
                <div>
                  <div className="font-medium text-[color:var(--text-1)] group-hover:text-[color:var(--green)] transition">{m.name}</div>
                  <div className="font-mono text-xs text-[color:var(--text-3)] mt-0.5">{m.params} · {m.downloads} · {m.family}</div>
                </div>
              </div>
              <div className="flex items-center gap-1.5 text-[color:var(--green)] font-mono text-xs shrink-0">
                <ShieldCheck size={13} /> {m.formats.length} {lang === "fr" ? "formats" : "formats"}
              </div>
            </div>
            <p className="mt-3 text-sm text-[color:var(--text-3)] line-clamp-2">{m.description}</p>
            <div className="mt-3 flex flex-wrap gap-1.5">
              {m.formats.slice(0, 6).map((slug) => {
                const f = FORMATS.find((x) => x.slug === slug);
                const color = f?.color ?? "#888";
                return <span key={slug} className="font-mono text-[10px] px-2 py-0.5 rounded border" style={{ borderColor: color + "55", color, background: color + "10" }}>{f?.name ?? slug}</span>;
              })}
              {m.formats.length > 6 && (
                <span className="font-mono text-[10px] px-2 py-0.5 rounded border border-[color:var(--border)] text-[color:var(--text-3)]">+{m.formats.length - 6}</span>
              )}
            </div>
          </button>
        ))}
      </div>

      {open && <ModelModal model={open} onClose={() => setOpen(null)} lang={lang} />}
    </PageStub>
  );
}

function ModelModal({ model, onClose, lang }: { model: Model; onClose: () => void; lang: "fr" | "en" }) {
  const [downloading, setDownloading] = useState<string | null>(null);
  const [done, setDone] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  const startDownload = (slug: string) => {
    setDownloading(slug);
    setTimeout(() => {
      setDownloading(null);
      setDone((d) => ({ ...d, [slug]: true }));
    }, 1600);
  };

  return (
    <div className="fixed inset-0 z-[60] grid place-items-center p-4 bg-black/70 backdrop-blur-sm animate-fade-in" onClick={onClose}>
      <div className="relative max-w-2xl w-full max-h-[88vh] overflow-y-auto rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-1)] p-7 animate-scale-in"
        onClick={(e) => e.stopPropagation()}>
        <button onClick={onClose} className="absolute top-4 right-4 p-1.5 rounded-md hover:bg-[color:var(--bg-3)] text-[color:var(--text-3)]"><X size={16} /></button>
        <div className="flex items-start gap-4">
          <BrandMark brand={model.org} size={48} />
          <div>
            <h2 className="t-h2 !text-2xl">{model.name}</h2>
            <div className="font-mono text-xs text-[color:var(--text-3)] mt-1">{model.params} · {model.downloads} downloads · {model.family}</div>
          </div>
        </div>
        <p className="mt-5 text-[color:var(--text-2)] leading-relaxed">{model.description}</p>

        <div className="mt-6 pt-5 border-t border-[color:var(--border)]">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mb-3">
            {lang === "fr" ? "Formats disponibles" : "Available formats"} · <span className="text-[color:var(--green)]"><ShieldCheck size={11} className="inline" /> certifiés ed25519</span>
          </div>
          <div className="space-y-2">
            {model.formats.map((slug) => {
              const f = FORMATS.find((x) => x.slug === slug);
              const color = f?.color ?? "#888";
              const size = mockSize(model.params, slug);
              return (
                <div key={slug} className="flex items-center justify-between gap-3 p-3 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-2)]">
                  <div className="flex items-center gap-3 min-w-0">
                    <span className="w-9 h-9 rounded-md grid place-items-center font-mono text-[10px] font-semibold shrink-0"
                      style={{ background: color + "20", color, border: `1px solid ${color}55` }}>
                      {(f?.name ?? slug).slice(0, 3).toUpperCase()}
                    </span>
                    <div className="min-w-0">
                      <div className="text-sm font-medium truncate">{f?.name ?? slug}</div>
                      <div className="font-mono text-[11px] text-[color:var(--text-3)]">{size} · {f?.ext ?? slug}</div>
                    </div>
                  </div>
                  <button onClick={() => startDownload(slug)} disabled={downloading === slug}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md font-mono text-xs transition shrink-0"
                    style={{
                      background: done[slug] ? "var(--green)" : color + "15",
                      color: done[slug] ? "var(--bg-0)" : color,
                      border: `1px solid ${done[slug] ? "var(--green)" : color + "55"}`,
                    }}>
                    {downloading === slug ? (
                      <>
                        <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                        {lang === "fr" ? "Démarrage…" : "Starting…"}
                      </>
                    ) : done[slug] ? (
                      <><CheckCircle2 size={12} /> {lang === "fr" ? "Lancé" : "Started"}</>
                    ) : (
                      <><Download size={12} /> {lang === "fr" ? "Télécharger" : "Download"}</>
                    )}
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

function mockSize(params: string, slug: string): string {
  const num = parseFloat(params);
  if (!num || isNaN(num)) return "—";
  const isB = params.includes("B");
  const bytes = (isB ? num : num / 1000) * 2; // base FP16
  const factor: Record<string, number> = { gguf: 0.55, awq: 0.3, gptq: 0.3, coreml: 0.6, tflite: 0.55, mlx: 1, mnn: 0.55, executorch: 0.6, onnx: 1, tensorrt: 1, openvino: 1, safetensors: 1, ncnn: 0.55 };
  const gb = bytes * (factor[slug] ?? 1);
  return gb >= 1 ? `${gb.toFixed(1)} Go` : `${(gb * 1024).toFixed(0)} Mo`;
}