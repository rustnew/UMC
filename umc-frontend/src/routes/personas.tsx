import { createFileRoute } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { Cpu, FlaskConical, ShieldCheck, Smartphone, Rocket, ArrowRight, CheckCircle2 } from "lucide-react";

export const Route = createFileRoute("/personas")({
  component: PersonasPage,
  head: () => ({
    meta: [
      { title: "Pour qui UMC ? — Cas d'usage par métier" },
      { name: "description", content: "Comment ingénieurs MLOps, chercheurs ML, industries régulées, devs mobile et startups IA utilisent UMC au quotidien." },
    ],
  }),
});

const PERSONAS = [
  {
    icon: Cpu, color: "#00FF94", tag: "01 · Cœur de cible",
    role: "Ingénieurs MLOps & équipes de déploiement IA",
    pain: "Un modèle PyTorch doit être converti en TensorRT (GPU prod), TFLite (mobile) et ONNX (cloud) — trois opérations, trois outils, trois jours chacun, zéro garantie que le converti se comporte comme l'original.",
    daily: [
      "Conversion PyTorch → TensorRT en une commande, avec δ < 1e-6 mesurée",
      "Pipelines CI/CD : conversion auto à chaque release modèle",
      "Rapport de conversion documenté tenseur par tenseur",
      "Détection automatique des opérateurs non-supportés avant déploiement",
    ],
    win: "Trois jours de plomberie → trois minutes. Suppression d'une source majeure de dette technique.",
    cta: "Atelier MLOps",
  },
  {
    icon: FlaskConical, color: "#38E1FF", tag: "02 · Recherche",
    role: "Chercheurs ML & équipes de recherche appliquée",
    pain: "Publier un modèle dans tous les formats que la communauté utilise (GGUF, SafeTensors, ONNX, CoreML) prend des semaines de travail après la publication du papier. La plupart abandonnent.",
    daily: [
      "Publier un modèle en 6+ formats simultanément depuis le CI/CD",
      "Joindre le rapport de conversion signé comme preuve de fidélité",
      "Donner accès à sa recherche aux utilisateurs Ollama, HuggingFace, Apple, NVIDIA",
      "Reproductibilité byte-perfect pour la review par les pairs",
    ],
    win: "Présence large dans l'écosystème vs. présence limitée à un seul format.",
    cta: "Voir le Hub",
  },
  {
    icon: ShieldCheck, color: "#B66BFF", tag: "03 · Enterprise",
    role: "Industries régulées — santé, finance, défense, automobile",
    pain: "Prouver à la FDA / EMA / ACPR / ISO 26262 que le modèle déployé est strictement identique au modèle validé. Aucune solution satisfaisante aujourd'hui sans UMC.",
    daily: [
      "Chaîne de provenance immutable de l'entraînement au déploiement",
      "Certificat ed25519 signé pour chaque conversion (preuve d'intégrité)",
      "Archivage 10 ans, replay byte-perfect sur demande du régulateur",
      "Conformité FDA 21 CFR Part 11, EU AI Act, MiFID II, Bâle III, ISO 26262",
    ],
    win: "Réponse directe à une obligation légale — justifie un contrat Enterprise 50–150 k€ / an.",
    cta: "Cas santé / finance",
  },
  {
    icon: Smartphone, color: "#FFC93C", tag: "04 · Mobile & Edge",
    role: "Développeurs d'applications mobiles & embarqué",
    pain: "Intégrer un modèle dans une app iOS, Android ou un device Qualcomm Snapdragon demande de comprendre CoreML, TFLite, QNN, ExecuTorch — alors que vous n'êtes pas expert ML.",
    daily: [
      "Une commande, détection automatique du format cible optimal",
      "Quantification automatique adaptée à la RAM de l'appareil cible",
      "Avertissement clair si le modèle est trop gros pour le device visé",
      "Export direct vers Xcode, Android Studio, ou SDK Qualcomm",
    ],
    win: "Le modèle fonctionne sur votre plateforme cible — sans devoir devenir expert en quantification.",
    cta: "Convertir pour mobile",
  },
  {
    icon: Rocket, color: "#FF4FD8", tag: "05 · Scale-up",
    role: "Startups IA & scale-ups en croissance rapide",
    pain: "Chaque semaine perdue sur de la plomberie de conversion coûte ≈ 2 000 € (ingénieur senior à 100 k€/an). Multiplier les équipes par format n'est pas viable.",
    daily: [
      "29 €/mois — décision triviale dès qu'un ingénieur économise 1 journée",
      "Pipelines automatisés multi-cibles (cloud GPU, mobile, API) en parallèle",
      "Pas besoin d'embaucher un spécialiste par format de déploiement",
      "Roadmap produit accélérée — déployer partout sans grossir l'équipe infra",
    ],
    win: "Croître sans exploser la masse salariale infra.",
    cta: "Calculer mon ROI",
  },
];

function PersonasPage() {
  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="max-w-3xl">
            <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">// Pour qui</div>
            <h1 className="t-h1 mt-3">Qui utilise UMC, et pour quoi faire.</h1>
            <p className="mt-5 text-lg text-[color:var(--text-2)] leading-relaxed">
              Cinq profils. Cinq problèmes très différents. Une seule plateforme qui les résout tous —
              parce que la fragmentation des formats IA touche chaque corps de métier différemment.
            </p>
          </div>

          <div className="mt-16 space-y-6">
            {PERSONAS.map((p, i) => (
              <article key={p.role} className="relative rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] overflow-hidden group hover:border-[color:var(--text-3)] transition">
                <div className="absolute inset-y-0 left-0 w-1" style={{ background: p.color, boxShadow: `0 0 24px ${p.color}` }} />
                <div className="grid lg:grid-cols-[1fr_1.4fr] gap-0">
                  <div className="p-8 lg:border-r border-[color:var(--border)]">
                    <div className="flex items-center gap-3">
                      <span className="w-12 h-12 rounded-xl grid place-items-center" style={{ background: `${p.color}18`, border: `1px solid ${p.color}55`, color: p.color }}>
                        <p.icon size={22} />
                      </span>
                      <div className="font-mono text-[10px] uppercase tracking-widest" style={{ color: p.color }}>{p.tag}</div>
                    </div>
                    <h2 className="mt-5 text-2xl font-display font-medium tracking-tight">{p.role}</h2>
                    <p className="mt-4 text-sm text-[color:var(--text-2)] leading-relaxed">{p.pain}</p>
                    <div className="mt-6 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-1)] p-4">
                      <div className="font-mono text-[10px] uppercase tracking-widest text-[color:var(--text-3)] mb-1.5">Bénéfice clé</div>
                      <div className="text-sm" style={{ color: p.color }}>{p.win}</div>
                    </div>
                  </div>
                  <div className="p-8 bg-[color:var(--bg-2)]">
                    <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mb-4">Au quotidien avec UMC</div>
                    <ul className="space-y-3">
                      {p.daily.map(d => (
                        <li key={d} className="flex gap-3 text-sm text-[color:var(--text-1)] leading-relaxed">
                          <CheckCircle2 size={16} className="mt-0.5 shrink-0" style={{ color: p.color }} />
                          <span>{d}</span>
                        </li>
                      ))}
                    </ul>
                    <a href="/signup" className="mt-7 inline-flex items-center gap-2 text-sm font-mono hover:gap-3 transition-all" style={{ color: p.color }}>
                      {p.cta} <ArrowRight size={14} />
                    </a>
                  </div>
                </div>
                <div aria-hidden className="absolute right-6 top-6 font-mono text-[11px] text-[color:var(--text-3)] opacity-60">0{i + 1} / 05</div>
              </article>
            ))}
          </div>

          <div className="mt-20 rounded-2xl border border-[color:var(--border)] p-10 text-center" style={{ background: "var(--gradient-hero)" }}>
            <h2 className="t-h2">Votre métier n'est pas listé ?</h2>
            <p className="mt-4 text-[color:var(--text-2)] max-w-xl mx-auto">
              UMC est neutre vis-à-vis du métier — n'importe quelle équipe qui déploie des modèles IA en bénéficie. Créez un compte gratuit et testez sur votre propre modèle.
            </p>
            <a href="/signup" className="mt-7 inline-flex items-center gap-2 px-6 py-3 rounded-lg text-[color:var(--bg-0)] font-semibold text-sm hover:brightness-110 transition"
              style={{ backgroundImage: "var(--gradient-brand)" }}>
              Créer mon compte gratuit <ArrowRight size={16} />
            </a>
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}