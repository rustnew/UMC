import { createFileRoute } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { ConversionUniverse } from "@/components/site/ConversionUniverse";
import { CompanyDropdown } from "@/components/site/CompanyDropdown";
import { Ticker } from "@/components/site/Ticker";
import { ROICalculator } from "@/components/site/ROICalculator";
import { ArrowRight, ShieldCheck, Zap, GitBranch, FileCheck, Cpu, Sparkles, Upload, Settings2, Download, AlertTriangle, CheckCircle2, XCircle, Code2, Layers, Globe2 } from "lucide-react";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)] relative">
      <Nav />
      <main className="relative">
        <Hero />
        <Ticker />
        <Problem />
        <BeforeAfter />
        <FfmpegParallel />
        <HowItWorks />
        <Product />
        <ROISection />
        <Guarantees />
        <section className="px-6 py-28 border-t border-[color:var(--border)]">
          <div className="max-w-7xl mx-auto">
            <div className="max-w-2xl mb-10">
              <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Écosystème</div>
              <h2 className="t-h1">Ces entreprises utilisent ces formats au quotidien.</h2>
              <p className="mt-4 text-[color:var(--text-2)]">UMC est l'intermédiaire neutre entre tous les acteurs. Aucun verrouillage matériel, aucune dépendance à un éditeur.</p>
            </div>
            <CompanyDropdown />
          </div>
        </section>
        <Formats />
        <CTA />
      </main>
      <Footer />
    </div>
  );
}

function Hero() {
  return (
    <section className="relative pt-32 pb-24 px-6 overflow-hidden">
      {/* aurora backdrop */}
      <div
        aria-hidden
        className="absolute inset-0 pointer-events-none"
        style={{ background: "var(--gradient-hero)" }}
      />
      <div
        aria-hidden
        className="absolute inset-x-0 -top-40 h-[520px] opacity-25 blur-3xl pointer-events-none aurora-bg"
        style={{ maskImage: "radial-gradient(ellipse at top, black, transparent 70%)", WebkitMaskImage: "radial-gradient(ellipse at top, black, transparent 70%)" }}
      />

      <div className="relative max-w-5xl mx-auto text-center">
        <div className="animate-float-up inline-flex items-center gap-2 rounded-full border border-[color:var(--border)] bg-[color:var(--bg-2)]/70 backdrop-blur px-3.5 py-1.5 font-mono text-[11px] text-[color:var(--text-2)]">
          <Sparkles size={12} className="text-[color:var(--green)]" />
          <span>Service en ligne · 31 formats · zéro installation</span>
        </div>

        <h1 className="t-hero mt-7 animate-float-up" style={{ animationDelay: "120ms" }}>
          The <span className="text-gradient-brand">ffmpeg</span>
          <br />
          of AI models.
        </h1>

        <p className="mt-7 text-lg text-[color:var(--text-2)] max-w-2xl mx-auto leading-relaxed animate-float-up" style={{ animationDelay: "240ms" }}>
          Une commande. 31 formats. 280+ chemins de conversion certifiés.
          Importez n'importe quel modèle, choisissez le format cible, téléchargez le résultat —
          avec un certificat <span className="text-[color:var(--green)]">ed25519</span> et <span className="text-[color:var(--green)]">δ &lt; 1e-6</span> garanti.
        </p>

        <div className="mt-9 flex flex-wrap justify-center gap-3 animate-float-up" style={{ animationDelay: "360ms" }}>
          <a
            href="/app"
            className="inline-flex items-center gap-2 px-6 py-3.5 rounded-lg text-[color:var(--bg-0)] font-semibold hover:brightness-110 transition shadow-[0_20px_50px_-15px_rgba(0,255,148,0.55)]"
            style={{ backgroundImage: "var(--gradient-brand)" }}
          >
            Convertir maintenant <ArrowRight size={16} />
          </a>
          <a
            href="/formats"
            className="inline-flex items-center gap-2 px-6 py-3.5 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] bg-[color:var(--bg-2)]/60 backdrop-blur transition font-mono text-sm"
          >
            Explorer les 31 formats
          </a>
        </div>

        <div className="mt-8 inline-flex flex-wrap justify-center gap-x-6 gap-y-2 font-mono text-xs text-[color:var(--text-3)] animate-float-up" style={{ animationDelay: "480ms" }}>
          {[
            ["31", "formats", "var(--green)"],
            ["280+", "chemins certifiés", "var(--cyan)"],
            ["4.2s", "Llama 8B", "var(--violet)"],
            ["δ < 1e-6", "précision", "var(--amber)"],
          ].map(([v, l, c]) => (
            <span key={l} className="inline-flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full" style={{ background: c, boxShadow: `0 0 8px ${c}` }} />
              <span className="text-[color:var(--text-1)] font-semibold">{v}</span>
              <span>{l}</span>
            </span>
          ))}
        </div>

        {/* convergence universe — front and center */}
        <div className="mt-16 animate-float-up" style={{ animationDelay: "600ms" }}>
          <ConversionUniverse />
          <p className="mt-5 text-sm text-[color:var(--text-3)] max-w-xl mx-auto">
            Tous les formats convergent vers UMC. Chaque éclat = une conversion en cours dans le monde.
          </p>
        </div>
      </div>
    </section>
  );
}

function Problem() {
  const items = [
    { k: "34 semaines", t: "Perdues chaque année", d: "Par équipe de 8 ingénieurs ML à recoder des pipelines de conversion." },
    { k: "65%", t: "Conversions silencieusement erronées", d: "Pertes d'information non détectées entre formats incompatibles." },
    { k: "800 Mo", t: "RAM pour convertir 810 Go", d: "Streaming mémoire mappé — modèles 100x plus grands que la RAM." },
  ];
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Le problème</div>
          <h2 className="t-h1">L'industrie IA est fragmentée à dessein.</h2>
          <p className="mt-4 text-[color:var(--text-2)]">
            NVIDIA, Apple, Google verrouillent leurs formats pour verrouiller leur matériel. UMC brise ce monopole.
          </p>
        </div>
        <div className="mt-10 grid md:grid-cols-3 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
          {items.map((i) => (
            <div key={i.t} className="bg-[color:var(--bg-2)] p-8">
              <div className="t-metric !text-4xl">{i.k}</div>
              <div className="mt-4 font-medium text-[color:var(--text-1)]">{i.t}</div>
              <div className="mt-2 text-sm text-[color:var(--text-3)] leading-relaxed">{i.d}</div>
            </div>
          ))}
        </div>

        <div className="mt-12 grid md:grid-cols-3 gap-6">
          {[
            {
              t: "Chaque éditeur impose son format",
              d: "PyTorch refuse de lire CoreML, TensorRT ignore MLX, ONNX perd les opérateurs custom. Vous êtes prisonnier de votre stack.",
            },
            {
              t: "Les conversions cassent silencieusement",
              d: "Un script bricolé convertit, mais perd 0,8% de précision. Vous le découvrez en production, sur un client.",
            },
            {
              t: "Aucune trace, aucun audit",
              d: "Quel script a généré ce .gguf ? Quelle quantification ? Avec quelle perte ? Personne ne le sait. Le modèle est une boîte noire.",
            },
          ].map((x) => (
            <div key={x.t} className="rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6">
              <AlertTriangle size={18} className="text-[color:var(--amber)]" />
              <div className="mt-3 font-medium text-[color:var(--text-1)]">{x.t}</div>
              <p className="mt-2 text-sm text-[color:var(--text-3)] leading-relaxed">{x.d}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function BeforeAfter() {
  const before = [
    "Cloner 6 dépôts GitHub différents et résoudre des conflits de versions Python",
    "Écrire 200 lignes de glue code par paire de formats (et 31 formats = 465 paires)",
    "Charger 70 Go en RAM pour convertir un modèle 70B, ou planter",
    "Vérifier la précision à la main, espérer que rien n'a dérivé",
    "Recommencer à chaque nouvelle version de PyTorch, ONNX, TensorRT",
    "Aucun audit, aucun certificat, aucun moyen de prouver l'intégrité",
  ];
  const after = [
    "Une seule commande, une seule URL, zéro installation locale",
    "280+ chemins certifiés couvrant tous les formats utilisés en production",
    "Streaming mmap : 800 Mo de RAM pour convertir 810 Go de poids",
    "Divergence δ < 1e-6 mesurée tenseur par tenseur, signée dans le certificat",
    "Mises à jour transparentes — l'API ne casse jamais",
    "Certificat ed25519 : preuve cryptographique de chaque conversion",
  ];
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl mb-12">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Avant / après</div>
          <h2 className="t-h1">Ce que change UMC, concrètement.</h2>
          <p className="mt-4 text-[color:var(--text-2)]">
            La conversion de modèles passe d'un projet de 3 semaines à une commande de 4 secondes.
          </p>
        </div>
        <div className="grid md:grid-cols-2 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
          <div className="bg-[color:var(--bg-2)] p-8">
            <div className="flex items-center gap-2 mb-5">
              <XCircle size={18} className="text-[color:var(--magenta)]" />
              <span className="font-mono text-sm uppercase tracking-widest text-[color:var(--magenta)]">Avant UMC</span>
            </div>
            <ul className="space-y-3">
              {before.map((b) => (
                <li key={b} className="flex gap-3 text-sm text-[color:var(--text-2)] leading-relaxed">
                  <span className="mt-1.5 w-1 h-1 rounded-full bg-[color:var(--magenta)] shrink-0" />
                  <span>{b}</span>
                </li>
              ))}
            </ul>
          </div>
          <div className="bg-[color:var(--bg-2)] p-8">
            <div className="flex items-center gap-2 mb-5">
              <CheckCircle2 size={18} className="text-[color:var(--green)]" />
              <span className="font-mono text-sm uppercase tracking-widest text-[color:var(--green)]">Avec UMC</span>
            </div>
            <ul className="space-y-3">
              {after.map((a) => (
                <li key={a} className="flex gap-3 text-sm text-[color:var(--text-1)] leading-relaxed">
                  <CheckCircle2 size={14} className="mt-0.5 text-[color:var(--green)] shrink-0" />
                  <span>{a}</span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </section>
  );
}

function FfmpegParallel() {
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-5xl mx-auto text-center">
        <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)]">// L'analogie</div>
        <h2 className="t-h1 mt-4">
          ffmpeg unifie la vidéo.<br />
          <span className="text-[color:var(--green)]">UMC unifie les modèles IA.</span>
        </h2>
        <p className="mt-5 text-[color:var(--text-2)] max-w-2xl mx-auto">
          Avant 2000, transcoder une vidéo entre DivX, MPEG-2 et QuickTime demandait trois logiciels et beaucoup de patience.
          ffmpeg a rendu cela invisible — une commande, tout fonctionne. En 2025, l'écosystème IA vit exactement la même fragmentation.
        </p>
        <div className="mt-12 grid md:grid-cols-2 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden text-left">
          {[
            { title: "ffmpeg (2000)", color: "var(--text-2)", rows: [
              ["Avant", "100+ codecs incompatibles"],
              ["Après", "1 commande, tout fonctionne"],
              ["Impact", "YouTube, Netflix, le web entier"],
            ]},
            { title: "UMC (2025)", color: "var(--green)", rows: [
              ["Avant", "31 formats IA isolés"],
              ["Après", "1 commande, tout converti"],
              ["Impact", "Tous les modèles, partout"],
            ]},
          ].map((c) => (
            <div key={c.title} className="bg-[color:var(--bg-2)] p-8">
              <div className="font-mono text-sm" style={{ color: c.color }}>{c.title}</div>
              <div className="mt-5 space-y-3">
                {c.rows.map(([k, v]) => (
                  <div key={k} className="grid grid-cols-[80px_1fr] gap-3 text-sm">
                    <span className="font-mono text-xs text-[color:var(--text-3)] uppercase pt-0.5">{k}</span>
                    <span className="text-[color:var(--text-1)]">{v}</span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className="mt-10 grid md:grid-cols-3 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden text-left">
          {[
            { k: "Conteneur universel", f: "MP4, MKV, AVI", u: "GGUF, ONNX, SafeTensors" },
            { k: "Codec / quantification", f: "H.264, AV1, VP9", u: "Q4_K_M, FP16, INT8" },
            { k: "Cible matérielle", f: "CPU, GPU, mobile, TV", u: "NVIDIA, Apple, Qualcomm, edge" },
          ].map((r) => (
            <div key={r.k} className="bg-[color:var(--bg-2)] p-6">
              <div className="font-mono text-[11px] uppercase tracking-widest text-[color:var(--text-3)]">{r.k}</div>
              <div className="mt-3 text-sm text-[color:var(--text-2)]">{r.f}</div>
              <div className="mt-1 text-sm text-[color:var(--green)] font-mono">→ {r.u}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function HowItWorks() {
  const steps = [
    {
      icon: Upload,
      n: "01",
      t: "Importez",
      d: "Glissez votre modèle dans le navigateur ou pointez vers une URL Hugging Face / S3. Aucune installation, aucun téléchargement local — UMC streame directement.",
      tag: "Tous formats acceptés",
    },
    {
      icon: Settings2,
      n: "02",
      t: "Configurez",
      d: "Choisissez le format cible et la précision. Notre matrice de compatibilité détecte automatiquement les chemins valides parmi 280+ paires certifiées.",
      tag: "Matrice 280+ chemins",
    },
    {
      icon: Download,
      n: "03",
      t: "Téléchargez",
      d: "Le modèle converti et son certificat ed25519 sont prêts. Export direct vers Hugging Face, S3, GCS ou téléchargement local.",
      tag: "Certificat inclus",
    },
  ];
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl mb-12">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Comment ça marche</div>
          <h2 className="t-h1">Trois étapes. Quatre secondes. Aucune installation.</h2>
          <p className="mt-4 text-[color:var(--text-2)]">
            UMC tourne entièrement dans le navigateur via WebAssembly et sur nos serveurs pour les modèles &gt; 4 Go. Vous gardez le contrôle.
          </p>
        </div>
        <div className="grid md:grid-cols-3 gap-6">
          {steps.map((s) => (
            <div key={s.n} className="relative rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7 overflow-hidden">
              <div aria-hidden className="absolute -right-6 -top-6 text-[120px] font-mono leading-none text-[color:var(--text-1)] opacity-[0.04] select-none">{s.n}</div>
              <div className="relative">
                <div className="inline-flex items-center justify-center w-10 h-10 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-1)]">
                  <s.icon size={18} className="text-[color:var(--green)]" />
                </div>
                <div className="mt-5 font-mono text-xs text-[color:var(--text-3)]">{s.n}</div>
                <h3 className="mt-1 t-h2 !text-xl">{s.t}</h3>
                <p className="mt-2.5 text-sm text-[color:var(--text-2)] leading-relaxed">{s.d}</p>
                <span className="mt-5 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border border-[color:var(--border)] font-mono text-[10px] uppercase tracking-widest text-[color:var(--text-2)]">
                  <span className="w-1 h-1 rounded-full bg-[color:var(--green)]" />{s.tag}
                </span>
              </div>
            </div>
          ))}
        </div>

        <pre className="mt-10 font-mono text-xs sm:text-sm bg-[color:var(--bg-0)] p-5 rounded-xl border border-[color:var(--border)] text-[color:var(--text-2)] overflow-x-auto">
{`$ umc convert llama-3.1-8b.safetensors --to gguf --quant Q4_K_M
→ streaming 16.0 GB via mmap... ok
→ converting 291 tensors (AVX-512)... 4.2s
→ verifying δ_max = 8.7e-3 (within Q4_K_M tolerance)... ok
→ signing with ed25519... ok

llama-3.1-8b.gguf      4.4 GB   ✓
llama-3.1-8b.umc.cert  512 B    ✓`}
        </pre>
      </div>
    </section>
  );
}

function Product() {
  const features = [
    { i: Layers, t: "31 formats, 280+ chemins", d: "GGUF, ONNX, SafeTensors, CoreML, TensorRT, TFLite, MLX, ExecuTorch… Toutes les paires utiles en production sont certifiées et testées en continu." },
    { i: Cpu, t: "Streaming mmap natif", d: "Convertissez des modèles 100× plus grands que votre RAM. Llama 405B se convertit avec 800 Mo de mémoire active." },
    { i: ShieldCheck, t: "Certificat ed25519", d: "Chaque sortie est signée. Hash des tenseurs, options, version du moteur — tout est auditable et reproductible." },
    { i: Code2, t: "API, CLI et UI", d: "Atelier web sans installation, CLI pour la CI/CD, API REST pour intégrer UMC à votre pipeline MLOps." },
    { i: Globe2, t: "Edge & souverain", d: "Déploiement on-premise ou cloud souverain UE. Aucun modèle ne quitte votre infrastructure si vous l'exigez." },
    { i: Zap, t: "Plus rapide que llama.cpp", d: "Pipeline parallèle, SIMD natif (AVX-512, NEON, SVE), zéro-copie. Mesuré sur Mistral 7B : 6× plus rapide que les outils existants." },
  ];
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl mb-12">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Le produit</div>
          <h2 className="t-h1">Ce que UMC fait, en détail.</h2>
          <p className="mt-4 text-[color:var(--text-2)]">
            Une plateforme unique pour importer, convertir, vérifier et distribuer vos modèles — sans dépendance à un éditeur.
          </p>
        </div>
        <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
          {features.map(({ i: Icon, t, d }) => (
            <div key={t} className="bg-[color:var(--bg-2)] p-7 hover:bg-[color:var(--bg-3)] transition">
              <Icon size={20} className="text-[color:var(--green)]" />
              <div className="mt-4 font-medium text-[color:var(--text-1)]">{t}</div>
              <p className="mt-2 text-sm text-[color:var(--text-3)] leading-relaxed">{d}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function ROISection() {
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl mb-12">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Calculateur ROI</div>
          <h2 className="t-h1">Combien votre équipe perd-elle par an ?</h2>
        </div>
        <ROICalculator />
      </div>
    </section>
  );
}

function Guarantees() {
  const items = [
    { i: ShieldCheck, t: "Zéro perte d'information", d: "Round-trip vérifié sur tous les tenseurs." },
    { i: Zap, t: "Précision bornée", d: "Divergence maximale δ < 1e-6 garantie." },
    { i: GitBranch, t: "Équivalence opérateurs", d: "Mapping certifié entre runtimes." },
    { i: Cpu, t: "Formats exploitables", d: "Compatible CPU, GPU, NPU, Edge." },
    { i: FileCheck, t: "Certificat ed25519", d: "Signature cryptographique de chaque conversion." },
  ];
  return (
    <section className="px-6 py-28 border-t border-[color:var(--border)]">
      <div className="max-w-7xl mx-auto">
        <div className="max-w-2xl mb-12">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// Garanties</div>
          <h2 className="t-h1">Conversions certifiées, pas approximées.</h2>
        </div>
        <div className="grid sm:grid-cols-2 lg:grid-cols-5 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
          {items.map(({ i: Icon, t, d }) => (
            <div key={t} className="bg-[color:var(--bg-2)] p-6 hover:bg-[color:var(--bg-3)] transition">
              <Icon size={18} className="text-[color:var(--green)]" />
              <div className="mt-3 font-medium text-sm">{t}</div>
              <div className="mt-1.5 text-xs text-[color:var(--text-3)] leading-relaxed">{d}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function Formats() {
  const rows = [
    ["GGUF", "ONNX", "SafeTensors", "PyTorch", "CoreML", "TensorRT", "TFLite", "ExecuTorch", "MLX", "JAX"],
    ["OpenVINO", "RKNN", "NCNN", "MNN", "Paddle", "MindSpore", "TVM", "GGML", "AWQ", "GPTQ"],
    ["Q4_K_M", "Q5_K_S", "Q8_0", "FP16", "BF16", "FP8", "INT8", "INT4", "F32", "F64"],
  ];
  return (
    <section className="py-28 border-t border-[color:var(--border)] overflow-hidden">
      <div className="max-w-7xl mx-auto px-6 mb-12">
        <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">// 31 formats</div>
        <h2 className="t-h1">Du serveur GPU à l'iPhone.</h2>
      </div>
      <div className="space-y-3">
        {rows.map((row, i) => (
          <div
            key={i}
            className="flex gap-3 animate-ticker"
            style={{ animationDuration: `${50 + i * 15}s`, animationDirection: i % 2 ? "reverse" : "normal" }}
          >
            {[...row, ...row, ...row].map((f, j) => (
              <span
                key={j}
                className="shrink-0 px-4 py-2.5 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-2)] font-mono text-sm text-[color:var(--text-2)] hover:border-[color:var(--green)] hover:text-[color:var(--text-1)] transition"
              >
                {f}
              </span>
            ))}
          </div>
        ))}
      </div>
      <div className="text-center mt-10">
        <a href="/formats" className="font-mono text-sm text-[color:var(--green)] hover:underline">
          Voir les 31 formats →
        </a>
      </div>
    </section>
  );
}

function CTA() {
  return (
    <section className="relative px-6 py-32 border-t border-[color:var(--border)] overflow-hidden">
      <div aria-hidden className="absolute inset-0 pointer-events-none opacity-[0.18] aurora-bg" />
      <div className="relative max-w-4xl mx-auto text-center">
        <h2 className="t-h1">Convertissez votre premier modèle en 4 secondes.</h2>
        <p className="mt-4 text-[color:var(--text-2)]">
          Aucune installation. 10 conversions gratuites par mois. Certificat inclus.
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-3">
          <a href="/app" className="inline-flex items-center gap-2 px-6 py-3.5 rounded-lg text-[color:var(--bg-0)] font-semibold hover:brightness-110 transition shadow-[0_20px_50px_-15px_rgba(0,255,148,0.55)]"
            style={{ backgroundImage: "var(--gradient-brand)" }}>
            Lancer une conversion <ArrowRight size={16} />
          </a>
          <a href="/pricing" className="inline-flex items-center gap-2 px-6 py-3.5 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] bg-[color:var(--bg-2)]/60 backdrop-blur transition">
            Voir les tarifs
          </a>
        </div>
      </div>
    </section>
  );
}
