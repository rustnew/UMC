import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { BRANDS, BrandMark, type BrandKey } from "@/lib/brands";
import { FORMATS } from "@/lib/formats";
import { getCompanyProfile, hasCompanyProfile } from "@/lib/company-profiles";
import { useTheme, t } from "@/lib/theme";
import { ArrowLeft, ArrowRight, Sparkles, Wrench } from "lucide-react";

export const Route = createFileRoute("/companies/$slug")({
  loader: ({ params }) => {
    if (!hasCompanyProfile(params.slug)) throw notFound();
    return { brand: params.slug as BrandKey };
  },
  component: CompanyPage,
  notFoundComponent: () => (
    <div className="min-h-screen grid place-items-center bg-[color:var(--bg-1)] text-[color:var(--text-2)]">
      Entreprise introuvable.
    </div>
  ),
  head: ({ params }) => {
    const slug = params.slug as string;
    const name = hasCompanyProfile(slug) ? BRANDS[slug as BrandKey].name : "Entreprise";
    return {
      meta: [
        { title: `${name} & UMC — Formats IA utilisés` },
        { name: "description", content: `Comment ${name} utilise les formats IA et comment UMC s'intègre dans son écosystème.` },
      ],
    };
  },
});

function CompanyPage() {
  const { brand } = Route.useLoaderData() as { brand: BrandKey };
  const { lang } = useTheme();
  const b = BRANDS[brand];
  const p = getCompanyProfile(brand);

  const createdFmts = p.created.map((s) => FORMATS.find((f) => f.slug === s)).filter(Boolean) as typeof FORMATS;
  const usedFmts = p.uses.map((s) => FORMATS.find((f) => f.slug === s)).filter(Boolean) as typeof FORMATS;

  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-28 pb-20 px-6">
        <article className="max-w-5xl mx-auto">
          <Link to="/companies" className="inline-flex items-center gap-1.5 font-mono text-xs text-[color:var(--text-3)] hover:text-[color:var(--text-1)]">
            <ArrowLeft size={12} /> {t({ fr: "Toutes les entreprises", en: "All companies" }, lang)}
          </Link>

          <header className="mt-6 flex items-center gap-5">
            <BrandMark brand={brand} size={64} />
            <div>
              <h1 className="t-h1 !text-4xl md:!text-5xl">{b.name}</h1>
              <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mt-1">
                {t({ fr: "Profil écosystème IA", en: "AI ecosystem profile" }, lang)}
              </div>
            </div>
          </header>

          <p className="mt-8 text-lg text-[color:var(--text-2)] leading-relaxed max-w-3xl">{p.bio[lang]}</p>

          {createdFmts.length > 0 && (
            <section className="mt-12">
              <div className="font-mono text-xs uppercase tracking-widest mb-3" style={{ color: b.color }}>
                <Sparkles size={12} className="inline -mt-0.5 mr-1" />
                {t({ fr: "// Formats créés par", en: "// Formats created by" }, lang)} {b.name}
              </div>
              <div className="grid sm:grid-cols-2 gap-4">
                {createdFmts.map((f) => (
                  <Link key={f.slug} to="/formats/$slug" params={{ slug: f.slug }}
                    className="rounded-xl border p-5 hover:border-[color:var(--text-3)] transition"
                    style={{ borderColor: f.color + "55", background: f.color + "0F" }}>
                    <div className="flex items-center justify-between">
                      <div className="font-medium text-lg" style={{ color: f.color }}>{f.name}</div>
                      <span className="font-mono text-[10px] text-[color:var(--text-3)]">{f.ext} · {f.year}</span>
                    </div>
                    <p className="mt-2 text-sm text-[color:var(--text-2)]">{f.why[lang]}</p>
                  </Link>
                ))}
              </div>
            </section>
          )}

          <section className="mt-12">
            <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mb-3">
              <Wrench size={12} className="inline -mt-0.5 mr-1" />
              {t({ fr: "// Formats utilisés au quotidien", en: "// Formats used daily" }, lang)}
            </div>
            <div className="flex flex-wrap gap-2">
              {usedFmts.map((f) => (
                <Link key={f.slug} to="/formats/$slug" params={{ slug: f.slug }}
                  className="inline-flex items-center gap-2 px-3 py-1.5 rounded-md border text-sm hover:translate-y-[-1px] transition"
                  style={{ borderColor: f.color + "55", color: f.color, background: f.color + "12" }}>
                  {f.name}
                </Link>
              ))}
            </div>
          </section>

          <section className="mt-12 grid md:grid-cols-2 gap-5">
            <div className="rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6">
              <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--text-3)] mb-3">
                {t({ fr: "Usage concret", en: "Concrete usage" }, lang)}
              </div>
              <p className="text-[color:var(--text-2)] leading-relaxed">{p.usage[lang]}</p>
            </div>
            <div className="rounded-2xl border p-6 relative overflow-hidden"
              style={{ borderColor: "var(--green)" + "55", background: "var(--green)" + "08" }}>
              <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">
                {t({ fr: "Rôle d'UMC dans cet écosystème", en: "UMC's role in this ecosystem" }, lang)}
              </div>
              <p className="text-[color:var(--text-2)] leading-relaxed">{p.umcRole[lang]}</p>
              <a href="/app" className="mt-5 inline-flex items-center gap-2 font-mono text-sm text-[color:var(--green)] hover:underline">
                {t({ fr: "Essayer une conversion", en: "Try a conversion" }, lang)} <ArrowRight size={14} />
              </a>
            </div>
          </section>
        </article>
      </main>
      <Footer />
    </div>
  );
}