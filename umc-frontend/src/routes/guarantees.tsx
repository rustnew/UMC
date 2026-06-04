import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { ShieldCheck, Zap, GitBranch, FileCheck, Cpu } from "lucide-react";

export const Route = createFileRoute("/guarantees")({ component: Page });

const ITEMS = [
  { icon: ShieldCheck, color: "#00FF94", title: "Zéro perte d'information",
    body: "Pour chaque conversion lossless (F32 ↔ F16 ↔ BF16, SafeTensors ↔ PyTorch ↔ GGUF), UMC effectue un round-trip et compare octet par octet. Si une seule différence est détectée, la conversion est rejetée.",
    proof: "round-trip · byte-equality" },
  { icon: Zap, color: "#38E1FF", title: "Précision bornée",
    body: "Pour chaque conversion lossy (quantification), UMC mesure la divergence maximale δ tenseur par tenseur. Le contrat : F32 → F16 garantit δ < 1e-6 ; F16 → INT8 garantit δ < tolérance théorique.",
    proof: "δ_max < 1e-6 (F32→F16)" },
  { icon: GitBranch, color: "#B66BFF", title: "Équivalence opérateurs",
    body: "Quand un opérateur n'existe pas tel quel dans le format cible (ex. GELU exact vs approx), UMC le décompose en primitives équivalentes mathématiquement, validées par tests de référence.",
    proof: "1 200+ tests d'équivalence" },
  { icon: Cpu, color: "#FFC93C", title: "Formats exploitables",
    body: "Une sortie UMC s'exécute immédiatement sur la cible déclarée : ONNX Runtime, llama.cpp, TensorRT, CoreML, TFLite, ExecuTorch, OpenVINO, etc. Aucune transformation manuelle requise.",
    proof: "CI cross-runtime sur 11 cibles" },
  { icon: FileCheck, color: "#FF4FD8", title: "Certificat ed25519",
    body: "Chaque conversion produit un certificat .umc.cert : hashes SHA-256 entrée/sortie, paramètres exacts, divergence mesurée, horodatage signé. Vérifiable hors-ligne avec n'importe quel outil ed25519.",
    proof: "ed25519 · attestation hors-ligne" },
];

function Page() {
  return (
    <PageStub
      eyebrow="Garanties"
      title="Cinq engagements contractuels, pas marketing."
      description="UMC ne convertit pas seulement — UMC certifie. Voici exactement ce que vous obtenez à chaque conversion."
    >
      <div className="space-y-4">
        {ITEMS.map(({ icon: Icon, ...g }) => (
          <article key={g.title} className="rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7 hover:border-[color:var(--text-3)] transition flex flex-col md:flex-row gap-6">
            <div className="md:w-48 shrink-0">
              <span className="w-12 h-12 rounded-xl inline-flex items-center justify-center"
                style={{ background: g.color + "20", border: `1px solid ${g.color}55`, color: g.color }}>
                <Icon size={22} />
              </span>
              <h3 className="t-h2 !text-xl mt-4">{g.title}</h3>
              <div className="mt-2 font-mono text-[11px]" style={{ color: g.color }}>{g.proof}</div>
            </div>
            <p className="flex-1 text-[color:var(--text-2)] leading-relaxed">{g.body}</p>
          </article>
        ))}
      </div>
    </PageStub>
  );
}