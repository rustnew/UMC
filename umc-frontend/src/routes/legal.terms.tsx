import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { useTheme, t } from "@/lib/theme";

export const Route = createFileRoute("/legal/terms")({
  component: Page,
  head: () => ({
    meta: [
      { title: "Conditions — UMC" },
      { name: "description", content: "Conditions générales d'utilisation du service UMC." },
    ],
  }),
});

function Page() {
  const { lang } = useTheme();
  return (
    <PageStub
      eyebrow={t({ fr: "Légal", en: "Legal" }, lang)}
      title={t({ fr: "Conditions d'utilisation", en: "Terms of service" }, lang)}
      description={t({
        fr: "Les règles d'usage du service UMC, dans un langage clair.",
        en: "The rules for using UMC, in plain language.",
      }, lang)}
    >
      <div className="text-[color:var(--text-2)] space-y-6 leading-relaxed">
        <p>{t({
          fr: "En utilisant UMC vous acceptez les présentes conditions. UMC fournit un service de conversion de modèles d'apprentissage automatique sans garantie sur les modèles que vous convertissez.",
          en: "By using UMC you accept these terms. UMC provides a model conversion service with no warranty over the models you convert.",
        }, lang)}</p>
        <p>{t({
          fr: "Usages interdits : contenus illégaux, modèles destinés à la surveillance de masse, à l'identification biométrique non consentie, ou à la génération de contenus pédopornographiques.",
          en: "Prohibited use: illegal content, models for mass surveillance, non-consenting biometric identification, or generation of CSAM.",
        }, lang)}</p>
        <p>{t({
          fr: "Le service est fourni « tel quel ». UMC garantit la fidélité numérique (δ < 1e-6 pour les conversions lossless) et signe chaque sortie. Aucune garantie ne porte sur les performances métier des modèles convertis.",
          en: "Service is provided as-is. UMC guarantees numerical fidelity (δ < 1e-6 for lossless conversions) and signs every output. No warranty covers business performance of converted models.",
        }, lang)}</p>
        <p>{t({ fr: "Droit applicable : France. Juridiction : Paris.", en: "Governing law: France. Jurisdiction: Paris." }, lang)}</p>
      </div>
    </PageStub>
  );
}