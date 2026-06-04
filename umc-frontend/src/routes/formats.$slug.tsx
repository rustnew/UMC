import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { FORMATS, type FormatDef, type Hardware } from "@/lib/formats";
import { BRANDS, BrandMark, type BrandKey } from "@/lib/brands";
import { useTheme, t } from "@/lib/theme";
import { ArrowLeft, ArrowRight, Cpu } from "lucide-react";

export const Route = createFileRoute("/formats/$slug")({
  loader: ({ params }) => {
    const f = FORMATS.find((x) => x.slug === params.slug);
    if (!f) throw notFound();
    return { format: f };
  },
  component: FormatPage,
  notFoundComponent: () => (
    <div className="min-h-screen grid place-items-center bg-[color:var(--bg-1)] text-[color:var(--text-2)]">
      Format introuvable.
    </div>
  ),
  head: ({ loaderData }) => ({
    meta: [
      { title: `${(loaderData as { format: FormatDef } | undefined)?.format.name ?? "Format"} — UMC` },
      { name: "description", content: "Format IA détaillé: historique, créateur, plateformes, exemple UMC." },
    ],
  }),
});

function FormatPage() {
  const { format: f } = Route.useLoaderData() as { format: FormatDef };
  const { lang } = useTheme();
  const creator = BRANDS[f.creator];

  const cli = `umc convert model.safetensors --to ${f.slug}`;

  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-28 pb-20 px-6">
        <article className="max-w-4xl mx-auto">
          <Link to="/formats" className="inline-flex items-center gap-1.5 font-mono text-xs text-[color:var(--text-3)] hover:text-[color:var(--text-1)]">
            <ArrowLeft size={12} /> {t({ fr: "Tous les formats", en: "All formats" }, lang)}
          </Link>

          <header className="mt-6 flex items-start gap-5">
            <span
              className="w-20 h-20 rounded-2xl grid place-items-center font-mono font-semibold text-lg shrink-0"
              style={{ background: f.color + "20", color: f.color, border: `1px solid ${f.color}55` }}>
              {f.name.slice(0, 3).toUpperCase()}
            </span>
            <div>
              <h1 className="t-h1 !text-4xl md:!text-5xl" style={{ color: f.color }}>{f.name}</h1>
              <div className="font-mono text-xs text-[color:var(--text-3)] mt-2">
                {f.ext} · {t({ fr: "depuis", en: "since" }, lang)} {f.year}
              </div>
            </div>
          </header>

          <section className="mt-10 grid md:grid-cols-2 gap-5">
            <InfoCard label={t({ fr: "Créateur", en: "Creator" }, lang)}>
              <Link to="/companies/$slug" params={{ slug: f.creator }} className="inline-flex items-center gap-2 group">
                <BrandMark brand={f.creator} size={28} />
                <span className="text-[color:var(--text-1)] group-hover:text-[color:var(--green)] transition">{creator.name} →</span>
              </Link>
            </InfoCard>

            <InfoCard label={t({ fr: "Plateformes cibles", en: "Target platforms" }, lang)}>
              <div className="flex flex-wrap gap-1.5">
                {f.hardware.map((h: Hardware) => (
                  <span key={h} className="inline-flex items-center gap-1 px-2 py-0.5 rounded border border-[color:var(--border)] font-mono text-[11px] text-[color:var(--text-2)]">
                    <Cpu size={10} /> {h}
                  </span>
                ))}
              </div>
            </InfoCard>
          </section>

          <Section title={t({ fr: "Histoire & raison d'être", en: "History & rationale" }, lang)}>
            <p>{f.why[lang]}</p>
          </Section>

          <Section title={t({ fr: "Cas d'usage principal", en: "Primary use case" }, lang)}>
            <p>{f.use[lang]}</p>
          </Section>

          <Section title={t({ fr: "Extensions de fichier", en: "File extensions" }, lang)}>
            <code className="font-mono text-sm bg-[color:var(--bg-2)] px-3 py-1 rounded border border-[color:var(--border)]">{f.ext}</code>
          </Section>

          <Section title={t({ fr: "Convertir avec UMC", en: "Convert with UMC" }, lang)}>
            <pre className="font-mono text-xs sm:text-sm bg-[color:var(--bg-0)] p-4 rounded-lg border border-[color:var(--border)] text-[color:var(--text-2)] overflow-x-auto">
{`# ${t({ fr: "Convertir vers", en: "Convert to" }, lang)} ${f.name}
$ ${cli} --quant Q4_K_M

# ${t({ fr: "Vérifier un binaire signé", en: "Verify a signed binary" }, lang)}
$ umc verify model${f.ext.split(" ")[0]} model.umc.cert`}
            </pre>
          </Section>

          <Section title={t({ fr: "Entreprises qui s'appuient sur ce format", en: "Companies relying on this format" }, lang)}>
            <div className="flex flex-wrap gap-2">
              {f.usedBy.map((b: BrandKey) => (
                <Link key={b} to="/companies/$slug" params={{ slug: b }}
                  className="inline-flex items-center gap-2 px-2.5 py-1 rounded-md border border-[color:var(--border)] hover:border-[color:var(--text-3)] transition">
                  <BrandMark brand={b} size={20} />
                  <span className="text-sm text-[color:var(--text-2)]">{BRANDS[b].name}</span>
                </Link>
              ))}
            </div>
          </Section>

          <div className="mt-12 flex flex-wrap gap-3">
            <a href={`/app?source=${f.slug}`}
              className="inline-flex items-center gap-2 px-5 py-3 rounded-lg font-medium text-[color:var(--bg-0)]"
              style={{ background: f.color }}>
              {t({ fr: "Convertir vers ce format", en: "Convert to this format" }, lang)} <ArrowRight size={15} />
            </a>
            <Link to="/formats" className="inline-flex items-center gap-2 px-5 py-3 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--text-3)] text-sm">
              {t({ fr: "Voir les 32 formats", en: "See all 32 formats" }, lang)}
            </Link>
          </div>
        </article>
      </main>
      <Footer />
    </div>
  );
}

function InfoCard({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-5">
      <div className="font-mono text-[10px] uppercase tracking-widest text-[color:var(--text-3)] mb-2">{label}</div>
      {children}
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="mt-10">
      <h2 className="t-h2 !text-xl text-[color:var(--text-1)] mb-3">{title}</h2>
      <div className="text-[color:var(--text-2)] leading-relaxed">{children}</div>
    </section>
  );
}