import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { useTheme, t } from "@/lib/theme";

export const Route = createFileRoute("/legal/privacy")({
  component: Page,
  head: () => ({
    meta: [
      { title: "Confidentialité — UMC" },
      { name: "description", content: "Politique de confidentialité de UMC. Aucune donnée de modèle stockée." },
    ],
  }),
});

function Page() {
  const { lang } = useTheme();
  return (
    <PageStub
      eyebrow={t({ fr: "Légal", en: "Legal" }, lang)}
      title={t({ fr: "Politique de confidentialité", en: "Privacy policy" }, lang)}
      description={t({
        fr: "UMC respecte la vie privée par défaut. Vos modèles ne sortent pas du navigateur en mode WASM, et ne sont jamais conservés sur nos serveurs.",
        en: "UMC is privacy-first by default. Your models never leave the browser in WASM mode, and are never stored on our servers.",
      }, lang)}
    >
      <div className="text-[color:var(--text-2)] space-y-6 leading-relaxed">
        <Section title={t({ fr: "Données collectées", en: "Data collected" }, lang)}
          body={t({
            fr: "Compte (email, nom d'affichage, organisation), logs techniques (IP tronquée, user-agent), conversions effectuées (méta-données uniquement, jamais les poids).",
            en: "Account (email, display name, organization), technical logs (truncated IP, user-agent), conversions performed (metadata only, never the weights).",
          }, lang)} />
        <Section title={t({ fr: "Conservation", en: "Retention" }, lang)}
          body={t({
            fr: "Modèles uploadés : effacés sous 30 minutes après conversion. Méta-données : 12 mois pour la facturation, supprimées sur demande.",
            en: "Uploaded models: erased within 30 minutes after conversion. Metadata: 12 months for billing, deleted on request.",
          }, lang)} />
        <Section title={t({ fr: "Vos droits", en: "Your rights" }, lang)}
          body={t({
            fr: "Accès, rectification, effacement, portabilité, opposition. Contact : privacy@umc.dev. Réponse sous 30 jours.",
            en: "Access, rectification, erasure, portability, opposition. Contact: privacy@umc.dev. Reply within 30 days.",
          }, lang)} />
        <Section title={t({ fr: "Hébergement", en: "Hosting" }, lang)}
          body={t({
            fr: "Cloudflare Workers (région EU) + base PostgreSQL Lovable Cloud (région EU). Aucun transfert hors UE.",
            en: "Cloudflare Workers (EU region) + Lovable Cloud PostgreSQL (EU region). No transfer outside the EU.",
          }, lang)} />
      </div>
    </PageStub>
  );
}

function Section({ title, body }: { title: string; body: string }) {
  return (
    <section>
      <h2 className="t-h2 !text-xl text-[color:var(--text-1)] mb-2">{title}</h2>
      <p>{body}</p>
    </section>
  );
}