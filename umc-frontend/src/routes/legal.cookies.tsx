import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { useTheme, t } from "@/lib/theme";

export const Route = createFileRoute("/legal/cookies")({
  component: Page,
  head: () => ({
    meta: [
      { title: "Cookies — UMC" },
      { name: "description", content: "Politique de gestion des cookies sur UMC." },
    ],
  }),
});

function Page() {
  const { lang } = useTheme();
  const reset = () => {
    localStorage.removeItem("umc.cookies.v1");
    location.reload();
  };
  return (
    <PageStub
      eyebrow={t({ fr: "Légal", en: "Legal" }, lang)}
      title={t({ fr: "Politique de cookies", en: "Cookie policy" }, lang)}
      description={t({
        fr: "Détail des cookies posés par UMC et comment révoquer votre consentement à tout moment.",
        en: "Detail of the cookies set by UMC and how to revoke consent at any time.",
      }, lang)}
    >
      <div className="prose prose-invert max-w-none text-[color:var(--text-2)] space-y-6">
        <section>
          <h2 className="t-h2 !text-xl text-[color:var(--text-1)]">
            {t({ fr: "Cookies essentiels", en: "Essential cookies" }, lang)}
          </h2>
          <p>
            {t({
              fr: "Nécessaires au fonctionnement du site (session, préférence de langue, choix de thème). Ils ne peuvent pas être désactivés.",
              en: "Required for the site to function (session, language preference, theme choice). They cannot be disabled.",
            }, lang)}
          </p>
          <ul className="font-mono text-xs mt-3 space-y-1 text-[color:var(--text-3)]">
            <li>umc-lang — {t({ fr: "préférence de langue", en: "language preference" }, lang)}</li>
            <li>umc-theme — {t({ fr: "préférence de thème", en: "theme preference" }, lang)}</li>
            <li>umc.cookies.v1 — {t({ fr: "choix de consentement", en: "consent choice" }, lang)}</li>
            <li>sb-* — {t({ fr: "session d'authentification", en: "auth session" }, lang)}</li>
          </ul>
        </section>
        <section>
          <h2 className="t-h2 !text-xl text-[color:var(--text-1)]">
            {t({ fr: "Mesure d'audience (optionnel)", en: "Analytics (optional)" }, lang)}
          </h2>
          <p>
            {t({
              fr: "Lorsque vous acceptez, nous mesurons de manière anonyme les pages les plus visitées pour améliorer UMC. Aucune donnée n'est revendue à des tiers.",
              en: "When you accept, we anonymously measure the most visited pages to improve UMC. No data is sold to third parties.",
            }, lang)}
          </p>
        </section>
        <section>
          <h2 className="t-h2 !text-xl text-[color:var(--text-1)]">
            {t({ fr: "Révoquer le consentement", en: "Revoke consent" }, lang)}
          </h2>
          <p>
            {t({
              fr: "Cliquez ci-dessous pour effacer votre choix actuel. Le bandeau réapparaîtra à votre prochaine visite.",
              en: "Click below to clear your current choice. The banner will appear again on next visit.",
            }, lang)}
          </p>
          <button
            onClick={reset}
            className="mt-4 px-4 py-2 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--green)] text-sm transition"
          >
            {t({ fr: "Réinitialiser mes préférences", en: "Reset my preferences" }, lang)}
          </button>
        </section>
      </div>
    </PageStub>
  );
}