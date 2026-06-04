import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { Heart, Banknote, Car, Shield, Film, Factory } from "lucide-react";

export const Route = createFileRoute("/use-cases")({ component: Page });

const CASES = [
  {
    icon: Heart, color: "#FF4FD8",
    sector: "Santé",
    title: "Imagerie médicale déployée sur scanners hétérogènes",
    body: "Un modèle de segmentation tumorale entraîné en PyTorch doit être déployé sur scanners GE Healthcare (TensorRT), Siemens (OpenVINO) et Philips (CoreML). UMC convertit en une session, génère un certificat ed25519 pour la documentation FDA / EU MDR, et garantit δ < 1e-5 sur chaque cible.",
    metric: "−84% de temps de validation",
    formats: ["PyTorch", "TensorRT", "OpenVINO", "CoreML"],
  },
  {
    icon: Banknote, color: "#00FF94",
    sector: "Finance",
    title: "Scoring de crédit auditable et conforme Bâle III",
    body: "Banque européenne tier-1 : modèle XGBoost + Transformer pour le scoring. Conversion vers ONNX pour service en multi-cloud, signature ed25519 archivée 10 ans, replay byte-perfect sur demande du régulateur.",
    metric: "100% reproductibilité",
    formats: ["PyTorch", "ONNX", "SafeTensors"],
  },
  {
    icon: Car, color: "#38E1FF",
    sector: "Automobile",
    title: "Vision ADAS embarquée multi-fournisseurs",
    body: "Constructeur premium : vision in-cabin sur GPU NVIDIA Drive Orin, détection d'objets sur iGPU Intel des passerelles, classification audio sur NPU Qualcomm. Trois cibles, trois formats, un seul pipeline UMC. Validation ISO 26262 ASIL-B.",
    metric: "3 cibles, 1 pipeline",
    formats: ["TensorRT", "OpenVINO", "QNN"],
  },
  {
    icon: Shield, color: "#B66BFF",
    sector: "Défense",
    title: "Modèles embarqués sur équipements classifiés",
    body: "Programme drone : conversion vers RKNN pour la vision Rockchip RK3588, vers ExecuTorch pour les wearables tactiques. Le certificat ed25519 prouve l'intégrité bout-en-bout — exigence souveraineté.",
    metric: "Conformité souveraineté",
    formats: ["RKNN", "ExecuTorch", "ONNX"],
  },
  {
    icon: Film, color: "#FFC93C",
    sector: "Divertissement",
    title: "Recommandation Netflix-style à l'échelle multi-cloud",
    body: "Plateforme de streaming : recommandation servie en parallèle sur AWS (ONNX Runtime), GCP (TFLite Serving) et Azure (ONNX + OpenVINO). UMC garantit que les trois cibles renvoient les mêmes recommandations à 1e-6 près.",
    metric: "3 clouds, 1 vérité",
    formats: ["PyTorch", "ONNX", "TFLite"],
  },
  {
    icon: Factory, color: "#FF7E2D",
    sector: "Industrie",
    title: "Maintenance prédictive en usine, déploiement edge massif",
    body: "Industriel 4.0 : 12 000 capteurs sur 47 sites. Modèle TimeSeries Transformer converti en TFLite Micro pour les ESP32 et en ONNX pour les passerelles industrielles. UMC orchestre l'upgrade flotte-wide via CI/CD.",
    metric: "12 000 nœuds upgradés en 4h",
    formats: ["PyTorch", "TFLite", "ONNX"],
  },
];

function Page() {
  return (
    <PageStub
      eyebrow="Cas d'usage"
      title="UMC en production, par industrie."
      description="Six secteurs régulés, six contraintes différentes, une seule plateforme universelle de conversion certifiée."
    >
      <div className="grid md:grid-cols-2 gap-5">
        {CASES.map(({ icon: Icon, ...c }) => (
          <article key={c.title} className="rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6 hover:border-[color:var(--text-3)] transition">
            <div className="flex items-center gap-3">
              <span className="w-11 h-11 rounded-xl grid place-items-center"
                style={{ background: c.color + "20", border: `1px solid ${c.color}55`, color: c.color }}>
                <Icon size={20} />
              </span>
              <div>
                <div className="font-mono text-[10px] uppercase tracking-widest" style={{ color: c.color }}>{c.sector}</div>
                <div className="font-medium mt-0.5">{c.title}</div>
              </div>
            </div>
            <p className="mt-4 text-sm text-[color:var(--text-2)] leading-relaxed">{c.body}</p>
            <div className="mt-5 flex items-center justify-between flex-wrap gap-3">
              <div className="flex flex-wrap gap-1.5">
                {c.formats.map((f) => (
                  <span key={f} className="font-mono text-[11px] px-2 py-1 rounded border border-[color:var(--border)] text-[color:var(--text-3)]">{f}</span>
                ))}
              </div>
              <span className="font-mono text-xs" style={{ color: c.color }}>{c.metric}</span>
            </div>
          </article>
        ))}
      </div>
    </PageStub>
  );
}