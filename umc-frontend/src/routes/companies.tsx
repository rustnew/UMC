import { createFileRoute } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { COMPANY_DEEP_DIVES } from "@/lib/formats";
import { FORMATS } from "@/lib/formats";
import { BRANDS, BrandMark } from "@/lib/brands";
import { Link } from "@tanstack/react-router";
import { useTheme } from "@/lib/theme";
import { ArrowRight, Check } from "lucide-react";

export const Route = createFileRoute("/companies")({
  component: Page,
  head: () => ({
    meta: [
      { title: "Entreprises — UMC" },
      { name: "description", content: "Comment Meta, Apple, Tesla, Spotify, Samsung, BMW et d'autres utilisent UMC au quotidien." },
    ],
  }),
});

function Page() {
  const { lang } = useTheme();
  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-28 pb-20 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--red)]">
            {lang === "fr" ? "// Entreprises" : "// Companies"}
          </div>
          <h1 className="t-h1 mt-3 max-w-3xl">
            {lang === "fr"
              ? "Comment les plus grandes entreprises convertissent leurs modèles tous les jours."
              : "How the biggest companies convert their models every day."}
          </h1>
          <p className="mt-4 text-lg text-[color:var(--text-2)] max-w-2xl">
            {lang === "fr"
              ? "Pour chaque secteur, un pipeline de conversion réel : du modèle d'entraînement au binaire déployé, avec les formats traversés et la tâche métier servie."
              : "For each sector, a real conversion pipeline: from the training model to the deployed binary, with formats traversed and the business task served."}
          </p>

          <div className="mt-12 grid md:grid-cols-2 gap-5">
            {COMPANY_DEEP_DIVES.map((c) => {
              const brand = BRANDS[c.brand];
              return (
                <Link key={c.brand} to="/companies/$slug" params={{ slug: c.brand }}
                  className="group relative block rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6 hover:border-[color:var(--text-3)] transition overflow-hidden">
                  <div
                    aria-hidden
                    className="absolute -top-20 -right-20 w-56 h-56 rounded-full opacity-20 blur-3xl pointer-events-none"
                    style={{ background: brand.color }}
                  />
                  <header className="flex items-start justify-between gap-3 relative">
                    <div className="flex items-center gap-3">
                      <BrandMark brand={c.brand} size={36} />
                      <div>
                        <h2 className="t-h2 !text-xl">{brand.name}</h2>
                        <div className="font-mono text-[10px] uppercase tracking-widest text-[color:var(--text-3)] mt-0.5">
                          {c.sector[lang]}
                        </div>
                      </div>
                    </div>
                  </header>

                  <ul className="mt-5 space-y-2 relative">
                    {c.daily[lang].map((d, i) => (
                      <li key={i} className="flex items-start gap-2 text-sm text-[color:var(--text-2)] leading-relaxed">
                        <Check size={14} className="mt-1 shrink-0" style={{ color: brand.color }} />
                        <span>{d}</span>
                      </li>
                    ))}
                  </ul>

                  <div className="mt-5 pt-4 border-t border-[color:var(--border)] relative">
                    <div className="font-mono text-[10px] uppercase tracking-widest text-[color:var(--text-3)] mb-2">
                      {lang === "fr" ? "Pipeline UMC" : "UMC pipeline"}
                    </div>
                    <div className="font-mono text-xs text-[color:var(--text-1)]">{c.flow[lang]}</div>
                  </div>

                  <div className="mt-4 flex flex-wrap gap-1.5 relative">
                    {c.formats.map((slug) => {
                      const f = FORMATS.find((x) => x.slug === slug);
                      if (!f) return null;
                      return (
                        <span key={slug}
                          className="font-mono text-[10px] px-2 py-0.5 rounded border"
                          style={{ borderColor: f.color + "55", color: f.color, background: f.color + "12" }}>
                          {f.name}
                        </span>
                      );
                    })}
                  </div>
                </Link>
              );
            })}
          </div>

          <section className="mt-20 rounded-2xl border border-[color:var(--border)] bg-gradient-to-br from-[color:var(--bg-2)] to-[color:var(--bg-1)] p-8">
            <h3 className="t-h2 !text-2xl">
              {lang === "fr" ? "Votre entreprise a un cas similaire ?" : "Does your company have a similar case?"}
            </h3>
            <p className="mt-3 text-[color:var(--text-2)] max-w-2xl">
              {lang === "fr"
                ? "Ouvrez l'atelier en un clic, importez votre modèle, choisissez la cible. Aucune installation, certificat ed25519 livré avec le binaire."
                : "Open the workshop in one click, upload your model, pick the target. Zero install, ed25519 certificate shipped with the binary."}
            </p>
            <a href="/app"
              className="mt-6 inline-flex items-center gap-2 px-5 py-3 rounded-lg font-medium text-[color:var(--bg-0)] shadow-[0_10px_30px_-10px_rgba(255,61,90,0.55)]"
              style={{ backgroundImage: "linear-gradient(135deg, var(--red), var(--orange))" }}>
              {lang === "fr" ? "Lancer une conversion" : "Run a conversion"} <ArrowRight size={15} />
            </a>
          </section>
        </div>
      </main>
      <Footer />
    </div>
  );
}