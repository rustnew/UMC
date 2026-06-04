import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { Check, X, Zap } from "lucide-react";
import { useState } from "react";

export const Route = createFileRoute("/pricing")({ component: Page });

function Page() {
  const [annual, setAnnual] = useState(true);
  const [modal, setModal] = useState<string | null>(null);

  const tiers = [
    { name: "Free", price: 0, suffix: "/mois", blurb: "Pour découvrir UMC", cta: "Commencer", features: ["10 conversions / mois", "Modèles ≤ 7B", "Certificat ed25519", "Hub public", "Communauté"] },
    { name: "Pay-as-you-go", price: 1, suffix: "€ – 3 € / conversion", blurb: "Sans abonnement, à l'usage", cta: "Acheter un crédit", features: ["1 € : modèles ≤ 7B", "2 € : modèles 7–70B", "3 € : modèles > 70B", "Certificat ed25519 inclus", "Aucun engagement"] },
    { name: "Pro", price: annual ? 15 : 19, suffix: "€/mois", blurb: "Pour équipes ML qui livrent", featured: true, cta: "Démarrer Pro", features: ["Conversions illimitées", "Files prioritaires GPU", "Hub privé", "API + webhooks", "Support prioritaire"] },
    { name: "Enterprise", price: null as number | null, suffix: "", blurb: "On-premise + SLA", cta: "Contacter l'équipe", features: ["Binaire on-premise", "SSO / SAML", "SLA 99.99%", "Audit & compliance", "Support dédié 24/7"] },
  ];

  return (
    <PageStub
      eyebrow="Tarifs"
      title="Un service universel. Quatre formules. Aucune installation."
      description="Plan gratuit pour découvrir. Paiement à l'usage pour les besoins ponctuels. Pro pour les équipes qui livrent. Enterprise pour les exigences sérieuses."
    >
      <div className="flex justify-center mb-10">
        <div className="inline-flex p-1 rounded-lg border border-[color:var(--border)] bg-[color:var(--bg-2)] text-sm font-mono">
          <button onClick={() => setAnnual(false)} className={`px-4 py-1.5 rounded-md transition ${!annual ? "bg-[color:var(--bg-4)] text-[color:var(--text-1)]" : "text-[color:var(--text-3)]"}`}>Mensuel</button>
          <button onClick={() => setAnnual(true)} className={`px-4 py-1.5 rounded-md transition ${annual ? "bg-[color:var(--bg-4)] text-[color:var(--text-1)]" : "text-[color:var(--text-3)]"}`}>Annuel <span className="text-[color:var(--green)] ml-1">-20%</span></button>
        </div>
      </div>

      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-4">
        {tiers.map((tier) => (
          <div key={tier.name} className={`p-6 rounded-xl border transition flex flex-col ${tier.featured ? "border-[color:var(--green)] bg-[color:var(--bg-2)] relative shadow-[0_0_40px_-15px_rgba(0,255,148,0.5)]" : "border-[color:var(--border)] bg-[color:var(--bg-2)]"}`}>
            {tier.featured && <div className="absolute -top-3 left-6 px-2 py-0.5 rounded font-mono text-[10px] uppercase bg-[color:var(--green)] text-[color:var(--bg-0)]">Populaire</div>}
            <div className="font-mono text-sm text-[color:var(--text-2)]">{tier.name}</div>
            <div className="mt-3 flex items-baseline gap-1 flex-wrap">
              {tier.price === null ? (
                <span className="t-h2 !text-3xl">Sur mesure</span>
              ) : (
                <>
                  <span className="t-h2 !text-4xl">{tier.price}{tier.name === "Pay-as-you-go" ? "" : "€"}</span>
                  <span className="text-xs text-[color:var(--text-3)]">{tier.suffix}</span>
                </>
              )}
            </div>
            <div className="mt-1 text-sm text-[color:var(--text-3)]">{tier.blurb}</div>
            <ul className="mt-5 space-y-2 flex-1">
              {tier.features.map((f) => (
                <li key={f} className="flex gap-2 text-sm text-[color:var(--text-2)]">
                  <Check size={14} className="text-[color:var(--green)] shrink-0 mt-0.5" />
                  {f}
                </li>
              ))}
            </ul>
            <button onClick={() => setModal(tier.name)}
              className={`mt-6 block w-full text-center py-2.5 rounded-lg font-medium text-sm transition ${tier.featured ? "bg-[color:var(--green)] text-[color:var(--bg-0)] hover:brightness-110" : "border border-[color:var(--border)] hover:border-[color:var(--text-3)]"}`}>
              {tier.cta}
            </button>
          </div>
        ))}
      </div>

      <section className="mt-20 rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-8">
        <div className="flex items-start gap-3">
          <Zap size={20} className="text-[color:var(--green)] mt-1" />
          <div>
            <h2 className="t-h2 !text-xl">Pourquoi payer pour UMC ?</h2>
            <p className="mt-3 text-[color:var(--text-2)] leading-relaxed">
              Maintenir 32 formats à jour est un travail à temps plein. Chaque mise à jour de TensorRT, CoreML ou TFLite peut casser des conversions existantes — une équipe dédiée veille, teste et certifie.
              Les serveurs GPU (A100, H100) qui produisent vos artefacts consomment de l'énergie réelle. La certification cryptographique exige une infrastructure HSM pour les clés ed25519 racines.
              Votre abonnement finance directement la R&D, l'infrastructure et le support — pour qu'UMC reste neutre, indépendant et fiable.
            </p>
          </div>
        </div>
      </section>

      {modal && <CheckoutModal tier={modal} onClose={() => setModal(null)} />}
    </PageStub>
  );
}

function CheckoutModal({ tier, onClose }: { tier: string; onClose: () => void }) {
  const [done, setDone] = useState(false);
  return (
    <div className="fixed inset-0 z-[60] grid place-items-center p-4 bg-black/70 backdrop-blur-sm animate-fade-in" onClick={onClose}>
      <div className="relative max-w-md w-full rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-1)] p-7 animate-scale-in" onClick={(e) => e.stopPropagation()}>
        <button onClick={onClose} className="absolute top-4 right-4 p-1.5 rounded-md hover:bg-[color:var(--bg-3)] text-[color:var(--text-3)]"><X size={16} /></button>
        {!done ? (
          <>
            <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">Plan {tier}</div>
            <h3 className="t-h2 !text-2xl mt-2">Démarrer en 30 secondes</h3>
            <form className="mt-6 space-y-3" onSubmit={(e) => { e.preventDefault(); setDone(true); }}>
              <input required type="email" placeholder="email@entreprise.com" className="w-full px-3 py-2.5 rounded-lg bg-[color:var(--bg-2)] border border-[color:var(--border)] text-sm focus:border-[color:var(--green)] outline-none" />
              <input required type="text" placeholder="Nom de l'organisation" className="w-full px-3 py-2.5 rounded-lg bg-[color:var(--bg-2)] border border-[color:var(--border)] text-sm focus:border-[color:var(--green)] outline-none" />
              <button type="submit" className="w-full py-2.5 rounded-lg bg-[color:var(--green)] text-[color:var(--bg-0)] font-medium text-sm hover:brightness-110 transition">Continuer vers le paiement</button>
              <p className="text-xs text-[color:var(--text-3)] text-center">Démo — aucun débit réel.</p>
            </form>
          </>
        ) : (
          <div className="text-center py-6">
            <Check size={36} className="text-[color:var(--green)] mx-auto" />
            <h3 className="t-h2 !text-xl mt-4">Merci !</h3>
            <p className="mt-2 text-sm text-[color:var(--text-2)]">Un email de confirmation vous sera envoyé. Vous pouvez dès maintenant utiliser l'atelier.</p>
            <a href="/app" className="mt-5 inline-block px-4 py-2 rounded-lg bg-[color:var(--green)] text-[color:var(--bg-0)] font-medium text-sm">Lancer l'atelier →</a>
          </div>
        )}
      </div>
    </div>
  );
}