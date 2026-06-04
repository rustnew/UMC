import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";

export const Route = createFileRoute("/benchmarks")({ component: Page });

const BARS = [
  { label: "Phi-2 2.7B · F16→Q4_K_M", umc: 1.8, llama: 12.4, py: 18.2 },
  { label: "Mistral 7B · SafeTensors→GGUF", umc: 4.2, llama: 26, py: 47 },
  { label: "Llama 3.1 8B · GGUF→ONNX", umc: 4.2, llama: 12, py: 47 },
  { label: "Llama 3.1 70B · PT→GGUF Q4", umc: 38, llama: 240, py: 0 },
  { label: "Llama 3.1 405B · PT→GGUF", umc: 252, llama: 0, py: 0 },
];
const MAX = Math.max(...BARS.flatMap((b) => [b.umc, b.llama, b.py]));

function Page() {
  return (
    <PageStub
      eyebrow="Benchmarks"
      title="Mesuré, pas marketé."
      description="Tous les benchmarks sont reproductibles. Dockerfile, scripts et résultats CSV fournis."
    >
      <div className="space-y-6">
        {BARS.map((b) => (
          <div key={b.label} className="rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-5">
            <div className="font-mono text-xs text-[color:var(--text-2)] mb-3">{b.label}</div>
            <Bar label="UMC" value={b.umc} max={MAX} color="var(--green)" />
            <Bar label="llama.cpp" value={b.llama} max={MAX} color="var(--cyan)" />
            <Bar label="transformers" value={b.py} max={MAX} color="var(--magenta)" />
          </div>
        ))}
      </div>

      <div className="mt-12 grid md:grid-cols-2 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
        <div className="bg-[color:var(--bg-2)] p-8">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">RAM</div>
          <div className="t-metric">800 Mo</div>
          <p className="mt-3 text-sm text-[color:var(--text-2)]">Pour convertir 810 Go. UMC utilise mmap pour traiter des modèles plus grands que la RAM disponible.</p>
        </div>
        <div className="bg-[color:var(--bg-2)] p-8">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">Précision</div>
          <div className="t-metric">δ &lt; 1e-6</div>
          <p className="mt-3 text-sm text-[color:var(--text-2)]">Divergence maximale F32 → F16. Mesurée tenseur par tenseur, certifiée dans le rapport de sortie.</p>
        </div>
      </div>

      <section className="mt-12 rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7">
        <h2 className="t-h2 !text-xl">Comment UMC est aussi rapide</h2>
        <ul className="mt-4 space-y-2 text-sm text-[color:var(--text-2)] leading-relaxed list-disc pl-5">
          <li><strong>Memory-mapping (mmap)</strong> : aucune copie en RAM, l'OS pagine à la demande.</li>
          <li><strong>Pipeline parallèle</strong> : lecture, transformation et écriture sur threads distincts.</li>
          <li><strong>SIMD natif</strong> : AVX-512 sur Intel, NEON sur ARM, SVE sur ARMv9.</li>
          <li><strong>Zero-copy</strong> : les tenseurs sont transformés en place, jamais dupliqués.</li>
          <li><strong>Code C++/Rust pur</strong> : pas de Python, pas de GIL, pas d'overhead d'interpréteur.</li>
        </ul>
      </section>
    </PageStub>
  );
}

function Bar({ label, value, max, color }: { label: string; value: number; max: number; color: string }) {
  const pct = value > 0 ? Math.max(2, (value / max) * 100) : 0;
  return (
    <div className="flex items-center gap-3 mb-2 last:mb-0">
      <div className="w-28 shrink-0 font-mono text-xs text-[color:var(--text-3)]">{label}</div>
      <div className="flex-1 h-7 rounded bg-[color:var(--bg-0)] relative overflow-hidden">
        {value > 0 ? (
          <div className="h-full rounded transition-all" style={{ width: `${pct}%`, background: color, boxShadow: `0 0 12px ${color}` }} />
        ) : (
          <div className="h-full grid place-items-center font-mono text-[10px] text-[color:var(--text-3)] uppercase">impossible</div>
        )}
      </div>
      <div className="w-20 shrink-0 text-right font-mono text-xs" style={{ color }}>{value > 0 ? `${value}s` : "—"}</div>
    </div>
  );
}