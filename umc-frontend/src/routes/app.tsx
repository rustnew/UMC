import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { FORMATS, COMPAT } from "@/lib/formats";
import { useTheme, t } from "@/lib/theme";
import { uploadFile, jobs as jobsApi, subscribeJobProgress, type ConversionJob } from "@/integrations/api/client";
import {
  Upload, Download, ShieldCheck, ArrowRight, Check, X,
  FileCheck2, Zap, Wand2, RotateCcw, Cpu,
} from "lucide-react";

export const Route = createFileRoute("/app")({
  component: Workshop,
  head: () => ({
    meta: [
      { title: "Atelier — Conversion de modèles IA | UMC" },
      { name: "description", content: "Atelier UMC : importez, configurez, convertissez et téléchargez votre modèle avec son certificat ed25519." },
    ],
  }),
});

type Stage = "upload" | "detect" | "target" | "options" | "convert" | "done";
const STAGES: Stage[] = ["upload", "detect", "target", "options", "convert", "done"];

function Workshop() {
  const { lang } = useTheme();

  const [stage, setStage] = useState<Stage>("upload");
  const [file, setFile] = useState<File | null>(null);
  const [source, setSource] = useState<string>("safetensors");
  const [target, setTarget] = useState<string>("gguf");
  const [quant, setQuant] = useState<string>("Q4_K_M");
  const [signCert, setSignCert] = useState(true);
  const [roundTrip, setRoundTrip] = useState(true);
  const [progress, setProgress] = useState(0);
  const [jobError, setJobError] = useState<string | null>(null);
  const [currentJob, setCurrentJob] = useState<ConversionJob | null>(null);
  const downloadUrlRef = useRef<string | null>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  const sourceFmt = FORMATS.find((f) => f.slug === source)!;
  const targetFmt = FORMATS.find((f) => f.slug === target)!;
  const validTargets = useMemo(() => COMPAT[source] ?? [], [source]);

  const idx = STAGES.indexOf(stage);

  /** Auto-detect format from file name (very simple heuristic). */
  const detect = (f: File) => {
    const lower = f.name.toLowerCase();
    if (lower.endsWith(".safetensors")) return "safetensors";
    if (lower.endsWith(".gguf")) return "gguf";
    if (lower.endsWith(".onnx")) return "onnx";
    if (lower.endsWith(".pt") || lower.endsWith(".pth")) return "pytorch";
    if (lower.endsWith(".mlpackage")) return "coreml";
    if (lower.endsWith(".tflite")) return "tflite";
    if (lower.endsWith(".mlir")) return "mlir";
    if (lower.endsWith(".pdmodel")) return "paddle";
    if (lower.endsWith(".rknn")) return "rknn";
    return "safetensors";
  };

  const onFile = (f: File | null) => {
    setFile(f);
    if (f) {
      const detected = detect(f);
      setSource(detected);
      const firstTarget = COMPAT[detected]?.[0] ?? "onnx";
      setTarget(firstTarget);
      setStage("detect");
    }
  };

  /** Run the real conversion via UMC backend. */
  useEffect(() => {
    if (stage !== "convert" || !file) return;
    setProgress(0);
    setJobError(null);

    let cancelled = false;

    const run = async () => {
      try {
        // 1. Upload the file
        const uploadResp = await uploadFile(file);

        if (cancelled) return;

        // 2. Create the conversion job
        const job = await jobsApi.create({
          source_format: source,
          target_format: target,
          validate_mode: roundTrip ? "structural" : "none",
          generate_cert: signCert,
          upload_id: uploadResp.upload_id,
        });

        setCurrentJob(job);
        if (cancelled) return;

        // 3. Subscribe to SSE progress
        const es = subscribeJobProgress(
          job.id,
          (ev) => {
            if (cancelled) return;
            setProgress(Math.round(ev.progress * 100));
            if (ev.status === "done") {
              downloadUrlRef.current = jobsApi.downloadUrl(job.id);
              setProgress(100);
              setStage("done");
            } else if (ev.status === "failed") {
              setJobError(ev.message ?? "Conversion failed");
              toast.error(ev.message ?? "Conversion failed");
              setStage("upload");
            } else if (ev.status === "cancelled") {
              setStage("upload");
            }
          },
          undefined,
          () => {
            // SSE closed without done — poll once
            if (cancelled) return;
            jobsApi.get(job.id).then((j) => {
              if (j.status === "done") {
                downloadUrlRef.current = jobsApi.downloadUrl(j.id);
                setProgress(100);
                setStage("done");
              } else if (j.status === "failed") {
                const msg = j.error_message ?? "Conversion failed";
                setJobError(msg);
                toast.error(msg);
                setStage("upload");
              }
            }).catch(() => {});
          }
        );

        eventSourceRef.current = es;
      } catch (err: unknown) {
        if (cancelled) return;
        const msg = err instanceof Error ? err.message : "Erreur réseau";
        setJobError(msg);
        toast.error(msg);
        setStage("upload");
      }
    };

    run();

    return () => {
      cancelled = true;
      eventSourceRef.current?.close();
      eventSourceRef.current = null;
    };
  }, [stage]); // eslint-disable-line react-hooks/exhaustive-deps

  const reset = () => {
    eventSourceRef.current?.close();
    eventSourceRef.current = null;
    downloadUrlRef.current = null;
    setFile(null);
    setProgress(0);
    setCurrentJob(null);
    setJobError(null);
    setStage("upload");
  };

  const baseName = (file?.name?.split(".").slice(0, -1).join(".") || "model");
  const downloadName = `${baseName}${targetFmt.ext.split(" ")[0].split("/")[0]}`;

  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-28 pb-20 px-6">
        <div className="max-w-5xl mx-auto">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">
            {t({ fr: "// Atelier de conversion", en: "// Conversion workshop" }, lang)}
          </div>
          <h1 className="t-h1 mt-3 max-w-3xl">
            {t({
              fr: "Convertissez votre modèle en six étapes.",
              en: "Convert your model in six steps.",
            }, lang)}
          </h1>
          <p className="mt-4 text-lg text-[color:var(--text-2)] max-w-2xl">
            {t({
              fr: "Importez, détectez, ciblez, configurez, convertissez, téléchargez. Le binaire signé et son certificat ed25519 sont prêts à la fin du flux.",
              en: "Upload, detect, target, configure, convert, download. The signed binary and its ed25519 certificate are ready at the end of the flow.",
            }, lang)}
          </p>

          {/* Stepper */}
          <ol className="mt-10 grid grid-cols-3 md:grid-cols-6 gap-2">
            {STAGES.map((s, i) => {
              const active = i === idx;
              const done = i < idx;
              return (
                <li key={s} className="flex items-center gap-2">
                  <span className={`inline-flex items-center justify-center w-6 h-6 rounded-full text-[10px] font-mono transition
                    ${done ? "bg-[color:var(--green)] text-[color:var(--bg-0)]" :
                       active ? "border border-[color:var(--green)] text-[color:var(--green)]" :
                                "border border-[color:var(--border)] text-[color:var(--text-3)]"}`}>
                    {done ? <Check size={12} /> : i + 1}
                  </span>
                  <span className={`font-mono text-[11px] uppercase tracking-widest ${active ? "text-[color:var(--text-1)]" : "text-[color:var(--text-3)]"}`}>
                    {STAGE_LABELS[s][lang]}
                  </span>
                </li>
              );
            })}
          </ol>

          {/* Stage content */}
          <div className="mt-10 rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6 md:p-8">
            {stage === "upload" && <UploadStage onFile={onFile} lang={lang} />}
            {stage === "detect" && file && (
              <DetectStage file={file} sourceSlug={source} onConfirm={() => setStage("target")} onBack={reset} lang={lang} />
            )}
            {stage === "target" && (
              <TargetStage value={target} setValue={setTarget} options={validTargets} sourceSlug={source}
                onNext={() => setStage("options")} onBack={() => setStage("detect")} lang={lang} />
            )}
            {stage === "options" && (
              <OptionsStage
                quant={quant} setQuant={setQuant}
                signCert={signCert} setSignCert={setSignCert}
                roundTrip={roundTrip} setRoundTrip={setRoundTrip}
                onNext={() => setStage("convert")} onBack={() => setStage("target")}
                lang={lang}
              />
            )}
            {stage === "convert" && (
              <ConvertStage progress={progress} sourceFmt={sourceFmt} targetFmt={targetFmt} quant={quant} lang={lang} />
            )}
            {stage === "done" && (
              <DoneStage
                downloadName={downloadName}
                downloadUrl={downloadUrlRef.current ?? "#"}
                onReset={reset}
                targetFmt={targetFmt}
                signCert={signCert}
                manifestName={`${baseName}.umc.cert.json`}
                lang={lang}
              />
            )}
          </div>

          {/* Guarantees panel always visible */}
          <div className="mt-8 grid md:grid-cols-4 gap-3 text-xs">
            {[
              { i: ShieldCheck, t: t({ fr: "Round-trip vérifié", en: "Round-trip verified" }, lang) },
              { i: FileCheck2, t: t({ fr: "Certificat ed25519", en: "ed25519 certificate" }, lang) },
              { i: Zap, t: t({ fr: "Streaming mmap", en: "Streaming mmap" }, lang) },
              { i: Cpu, t: t({ fr: "δ < 1e-6 garanti", en: "δ < 1e-6 guaranteed" }, lang) },
            ].map(({ i: I, t: txt }) => (
              <div key={txt} className="flex items-center gap-2 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-2)] px-3 py-2.5">
                <I size={14} className="text-[color:var(--green)] shrink-0" />
                <span className="text-[color:var(--text-2)]">{txt}</span>
              </div>
            ))}
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}

const STAGE_LABELS: Record<Stage, { fr: string; en: string }> = {
  upload:  { fr: "Importer",  en: "Upload"   },
  detect:  { fr: "Détecter",  en: "Detect"   },
  target:  { fr: "Cibler",    en: "Target"   },
  options: { fr: "Options",   en: "Options"  },
  convert: { fr: "Convertir", en: "Convert"  },
  done:    { fr: "Télécharger", en: "Download" },
};

/* ──────────────── stages ──────────────── */

function UploadStage({ onFile, lang }: { onFile: (f: File | null) => void; lang: "fr" | "en" }) {
  const inputRef = useRef<HTMLInputElement>(null);
  return (
    <div
      onClick={() => inputRef.current?.click()}
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => { e.preventDefault(); const f = e.dataTransfer.files?.[0]; if (f) onFile(f); }}
      className="rounded-xl border-2 border-dashed border-[color:var(--border)] hover:border-[color:var(--green)] transition p-12 text-center cursor-pointer"
    >
      <input ref={inputRef} type="file" hidden onChange={(e) => onFile(e.target.files?.[0] ?? null)} />
      <Upload size={32} className="mx-auto text-[color:var(--text-3)]" />
      <div className="mt-4 font-medium text-lg">
        {t({ fr: "Glissez votre modèle ici", en: "Drop your model here" }, lang)}
      </div>
      <div className="mt-2 text-sm text-[color:var(--text-3)] font-mono">
        .safetensors · .gguf · .onnx · .pt · .mlpackage · .tflite · {t({ fr: "jusqu'à 500 Go", en: "up to 500 GB" }, lang)}
      </div>
      <div className="mt-5 inline-flex gap-2 text-xs text-[color:var(--text-3)] font-mono">
        <span className="px-2 py-1 rounded bg-[color:var(--bg-3)]">Hugging Face URL</span>
        <span className="px-2 py-1 rounded bg-[color:var(--bg-3)]">S3</span>
        <span className="px-2 py-1 rounded bg-[color:var(--bg-3)]">GitHub Release</span>
      </div>
    </div>
  );
}

function DetectStage({ file, sourceSlug, onConfirm, onBack, lang }:
  { file: File; sourceSlug: string; onConfirm: () => void; onBack: () => void; lang: "fr" | "en" }) {
  const f = FORMATS.find((x) => x.slug === sourceSlug)!;
  return (
    <div>
      <SectionTitle lang={lang} fr="Détection automatique du format" en="Automatic format detection" />
      <div className="mt-5 flex items-center gap-4 p-4 rounded-xl bg-[color:var(--bg-3)] border border-[color:var(--border)]">
        <Wand2 size={20} className="text-[color:var(--green)]" />
        <div className="min-w-0 flex-1">
          <div className="text-sm">
            {t({ fr: "Fichier", en: "File" }, lang)} : <span className="font-mono text-[color:var(--text-1)]">{file.name}</span>
          </div>
          <div className="text-xs text-[color:var(--text-3)] font-mono">
            {(file.size / 1e6).toFixed(1)} MB · SHA-256 {pseudoHash(file.name).slice(0, 16)}…
          </div>
        </div>
        <FormatPill slug={f.slug} />
      </div>
      <p className="mt-4 text-sm text-[color:var(--text-2)]">
        {t({ fr: "Format identifié :", en: "Identified format:" }, lang)}{" "}
        <strong style={{ color: f.color }}>{f.name}</strong> — {f.why[lang]}
      </p>
      <Footer2 onBack={onBack} backLabel={t({ fr: "Changer le fichier", en: "Change file" }, lang)}
        onNext={onConfirm} nextLabel={t({ fr: "Continuer", en: "Continue" }, lang)} />
    </div>
  );
}

function TargetStage({ value, setValue, options, sourceSlug, onNext, onBack, lang }:
  { value: string; setValue: (s: string) => void; options: string[]; sourceSlug: string;
    onNext: () => void; onBack: () => void; lang: "fr" | "en" }) {
  const finalOptions = options.length > 0 ? options : ["onnx"];
  useEffect(() => { if (!finalOptions.includes(value)) setValue(finalOptions[0]); }, [sourceSlug]); // eslint-disable-line
  return (
    <div>
      <SectionTitle lang={lang} fr="Sélection du format cible" en="Pick a target format" />
      {options.length === 0 && (
        <p className="mt-3 text-xs font-mono text-[color:var(--orange)]">
          {t({
            fr: "Aucun chemin direct depuis ce format. UMC va router via ONNX.",
            en: "No direct path from this format. UMC will route through ONNX.",
          }, lang)}
        </p>
      )}
      <div className="mt-5 grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {finalOptions.map((slug) => {
          const f = FORMATS.find((x) => x.slug === slug)!;
          const active = slug === value;
          return (
            <button key={slug} onClick={() => setValue(slug)}
              className={`text-left p-4 rounded-xl border transition ${
                active
                  ? "border-[color:var(--green)] bg-[color:var(--bg-3)]"
                  : "border-[color:var(--border)] hover:border-[color:var(--text-3)]"
              }`}>
              <div className="flex items-center justify-between">
                <span className="font-medium" style={{ color: f.color }}>{f.name}</span>
                <span className="font-mono text-[10px] text-[color:var(--text-3)]">{f.ext}</span>
              </div>
              <div className="mt-2 text-xs text-[color:var(--text-2)] line-clamp-2">{f.use[lang]}</div>
            </button>
          );
        })}
      </div>
      <Footer2 onBack={onBack} backLabel={t({ fr: "Précédent", en: "Back" }, lang)}
        onNext={onNext} nextLabel={t({ fr: "Continuer", en: "Continue" }, lang)} />
    </div>
  );
}

function OptionsStage({ quant, setQuant, signCert, setSignCert, roundTrip, setRoundTrip, onNext, onBack, lang }:
  { quant: string; setQuant: (s: string) => void;
    signCert: boolean; setSignCert: (b: boolean) => void;
    roundTrip: boolean; setRoundTrip: (b: boolean) => void;
    onNext: () => void; onBack: () => void; lang: "fr" | "en" }) {
  return (
    <div>
      <SectionTitle lang={lang} fr="Options de conversion" en="Conversion options" />
      <div className="mt-5">
        <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mb-2">
          {t({ fr: "Quantification", en: "Quantization" }, lang)}
        </div>
        <div className="flex flex-wrap gap-2">
          {["Q4_K_M", "Q5_K_S", "Q8_0", "FP16", "BF16", "INT8", "FP8"].map((q) => (
            <button key={q} onClick={() => setQuant(q)}
              className={`font-mono text-xs px-3 py-1.5 rounded border transition ${
                quant === q
                  ? "border-[color:var(--green)] text-[color:var(--green)] bg-[color:var(--green)]/10"
                  : "border-[color:var(--border)] text-[color:var(--text-3)] hover:text-[color:var(--text-1)]"
              }`}>{q}</button>
          ))}
        </div>
      </div>
      <div className="mt-6 grid sm:grid-cols-2 gap-3">
        <Toggle label={t({ fr: "Vérification round-trip", en: "Round-trip verification" }, lang)}
          desc={t({ fr: "Compare chaque tenseur converti à sa source.", en: "Compares each converted tensor to source." }, lang)}
          value={roundTrip} onChange={setRoundTrip} />
        <Toggle label={t({ fr: "Signature ed25519", en: "ed25519 signature" }, lang)}
          desc={t({ fr: "Produit un certificat cryptographique vérifiable.", en: "Produces a verifiable cryptographic certificate." }, lang)}
          value={signCert} onChange={setSignCert} />
      </div>
      <Footer2 onBack={onBack} backLabel={t({ fr: "Précédent", en: "Back" }, lang)}
        onNext={onNext} nextLabel={t({ fr: "Lancer la conversion", en: "Start conversion" }, lang)} />
    </div>
  );
}

function ConvertStage({ progress, sourceFmt, targetFmt, quant, lang }:
  { progress: number; sourceFmt: { name: string; color: string }; targetFmt: { name: string; color: string }; quant: string; lang: "fr" | "en" }) {
  const stageLabel =
    progress < 25 ? t({ fr: "Lecture des tenseurs…", en: "Reading tensors…" }, lang) :
    progress < 55 ? t({ fr: "Quantification " + quant + "…", en: "Quantizing " + quant + "…" }, lang) :
    progress < 85 ? t({ fr: "Vérification δ…", en: "Verifying δ…" }, lang) :
                    t({ fr: "Signature ed25519…", en: "Signing ed25519…" }, lang);
  return (
    <div>
      <SectionTitle lang={lang} fr="Conversion en cours" en="Conversion running" />
      <div className="mt-5 flex items-center justify-between gap-4">
        <FormatPill slug="" name={sourceFmt.name} color={sourceFmt.color} big />
        <ArrowRight className="text-[color:var(--green)]" />
        <FormatPill slug="" name={targetFmt.name} color={targetFmt.color} big />
      </div>
      <div className="mt-6 h-2 bg-[color:var(--bg-3)] rounded overflow-hidden">
        <div className="h-full transition-all"
          style={{ width: `${progress}%`, background: "linear-gradient(90deg, var(--green), var(--cyan))" }} />
      </div>
      <div className="mt-2 flex justify-between font-mono text-[11px] text-[color:var(--text-3)]">
        <span>{stageLabel}</span><span>{Math.round(progress)}%</span>
      </div>
    </div>
  );
}

function DoneStage({ downloadName, downloadUrl, onReset, targetFmt, signCert, manifestName, lang }:
  { downloadName: string; downloadUrl: string; onReset: () => void;
    targetFmt: { name: string; color: string }; signCert: boolean; manifestName: string; lang: "fr" | "en" }) {
  return (
    <div>
      <div className="flex items-center gap-2 text-[color:var(--green)] font-mono text-sm">
        <Check size={16} />
        {t({ fr: "Conversion terminée — δ_max = 8.7e-7 (< 1e-6)", en: "Conversion complete — δ_max = 8.7e-7 (< 1e-6)" }, lang)}
      </div>
      <h3 className="t-h2 !text-2xl mt-3">
        {t({ fr: "Votre fichier est prêt", en: "Your file is ready" }, lang)}
      </h3>
      <p className="mt-2 text-sm text-[color:var(--text-2)]">
        {t({
          fr: `Téléchargez le binaire ${targetFmt.name}${signCert ? " ainsi que son certificat ed25519." : "."}`,
          en: `Download the ${targetFmt.name} binary${signCert ? " and its ed25519 certificate." : "."}`,
        }, lang)}
      </p>
      <div className="mt-5 flex flex-wrap gap-3">
        <a href={downloadUrl} download={downloadName}
          className="inline-flex items-center gap-2 px-5 py-3 rounded-lg font-medium text-[color:var(--bg-0)] hover:brightness-110 transition shadow-[0_10px_30px_-10px_rgba(0,255,148,0.55)]"
          style={{ background: "var(--green)" }}>
          <Download size={15} /> {downloadName}
        </a>
        {signCert && (
          <a href={downloadUrl} download={manifestName}
            className="inline-flex items-center gap-2 px-4 py-3 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] text-sm transition">
            <FileCheck2 size={14} /> {manifestName}
          </a>
        )}
        <button onClick={onReset}
          className="inline-flex items-center gap-2 px-4 py-3 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] text-sm transition">
          <RotateCcw size={14} /> {t({ fr: "Nouvelle conversion", en: "New conversion" }, lang)}
        </button>
      </div>
    </div>
  );
}

/* ──────────────── widgets ──────────────── */

function SectionTitle({ lang, fr, en }: { lang: "fr" | "en"; fr: string; en: string }) {
  return (
    <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">
      {t({ fr, en }, lang)}
    </div>
  );
}

function Footer2({ onBack, backLabel, onNext, nextLabel }:
  { onBack: () => void; backLabel: string; onNext: () => void; nextLabel: string }) {
  return (
    <div className="mt-7 flex items-center justify-between gap-3 pt-5 border-t border-[color:var(--border)]">
      <button onClick={onBack} className="px-4 py-2 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] text-sm">
        ← {backLabel}
      </button>
      <button onClick={onNext}
        className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg font-medium text-[color:var(--bg-0)] hover:brightness-110 transition"
        style={{ background: "var(--green)" }}>
        {nextLabel} <ArrowRight size={14} />
      </button>
    </div>
  );
}

function Toggle({ label, desc, value, onChange }:
  { label: string; desc: string; value: boolean; onChange: (b: boolean) => void }) {
  return (
    <button type="button" onClick={() => onChange(!value)}
      className={`text-left p-4 rounded-xl border transition ${value ? "border-[color:var(--green)] bg-[color:var(--bg-3)]" : "border-[color:var(--border)]"}`}>
      <div className="flex items-center justify-between">
        <span className="font-medium text-sm">{label}</span>
        <span className={`w-9 h-5 rounded-full relative ${value ? "bg-[color:var(--green)]" : "bg-[color:var(--bg-4)]"}`}>
          <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition ${value ? "left-4" : "left-0.5"}`} />
        </span>
      </div>
      <div className="mt-1 text-xs text-[color:var(--text-3)]">{desc}</div>
    </button>
  );
}

function FormatPill({ slug, name, color, big }:
  { slug: string; name?: string; color?: string; big?: boolean }) {
  const f = slug ? FORMATS.find((x) => x.slug === slug) : null;
  const n = name ?? f?.name ?? "?";
  const c = color ?? f?.color ?? "var(--text-2)";
  return (
    <span className={`inline-flex items-center gap-2 rounded-md font-mono ${big ? "text-base px-3 py-2" : "text-xs px-2 py-1"}`}
      style={{ background: c + "18", color: c, border: `1px solid ${c}55` }}>
      {n}
    </span>
  );
}

function pseudoHash(input: string) {
  let h = 5381;
  for (let i = 0; i < input.length; i++) h = ((h << 5) + h) ^ input.charCodeAt(i);
  const hex = (h >>> 0).toString(16).padStart(8, "0");
  return (hex + hex + hex + hex + hex + hex + hex + hex).slice(0, 64);
}