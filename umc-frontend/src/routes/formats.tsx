import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { FORMATS } from "@/lib/formats";
import { BRANDS, BrandMark } from "@/lib/brands";
import { Link } from "@tanstack/react-router";
import { CompanyDropdown } from "@/components/site/CompanyDropdown";
import { useTheme } from "@/lib/theme";
import { Cpu } from "lucide-react";

export const Route = createFileRoute("/formats")({ component: Page });

function Page() {
  const { lang } = useTheme();
  return (
    <PageStub
      eyebrow={lang === "fr" ? "Catalogue" : "Catalog"}
      title={lang === "fr" ? "31 formats de modèles IA, un seul standard." : "31 AI model formats, one standard."}
      description={lang === "fr"
        ? "Chaque format a une raison d'exister, un créateur, un matériel cible. UMC les comprend tous et certifie les conversions entre eux."
        : "Every format has a reason to exist, a creator, a target hardware. UMC speaks all of them and certifies conversions between them."}
    >
      <div className="grid md:grid-cols-2 gap-5">
        {FORMATS.map((f) => {
          const creator = BRANDS[f.creator];
          return (
            <Link key={f.slug} to="/formats/$slug" params={{ slug: f.slug }}
              className="block rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-6 hover:border-[color:var(--text-3)] transition">
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-center gap-3">
                  <span className="w-12 h-12 rounded-xl grid place-items-center font-mono font-semibold text-sm"
                    style={{ background: f.color + "20", color: f.color, border: `1px solid ${f.color}55` }}>
                    {f.name.slice(0, 3).toUpperCase()}
                  </span>
                  <div>
                    <div className="text-lg font-medium" style={{ color: f.color }}>{f.name}</div>
                    <div className="font-mono text-[11px] text-[color:var(--text-3)]">{f.ext} · {f.year}</div>
                  </div>
                </div>
                <div className="flex items-center gap-1.5 text-[10px] font-mono text-[color:var(--text-3)] uppercase tracking-widest">
                  <Cpu size={11} /> {f.hardware.join(" · ")}
                </div>
              </div>

              <div className="mt-5">
                <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)] tracking-widest">
                  {lang === "fr" ? "Créé par" : "Created by"}
                </div>
                <div className="mt-1.5"><BrandMark brand={f.creator} withName size={26} /></div>
              </div>

              <div className="mt-4">
                <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)] tracking-widest">
                  {lang === "fr" ? "Pourquoi ce format existe" : "Why it exists"}
                </div>
                <p className="mt-1.5 text-sm text-[color:var(--text-2)] leading-relaxed">{f.why[lang]}</p>
              </div>

              <div className="mt-4">
                <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)] tracking-widest">
                  {lang === "fr" ? "Cas d'usage" : "Use cases"}
                </div>
                <p className="mt-1.5 text-sm text-[color:var(--text-2)] leading-relaxed">{f.use[lang]}</p>
              </div>

              <div className="mt-5 pt-4 border-t border-[color:var(--border)]">
                <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)] tracking-widest mb-2">
                  {lang === "fr" ? "Utilisé par" : "Used by"}
                </div>
                <div className="flex flex-wrap gap-2">
                  {f.usedBy.map((b) => <BrandMark key={b} brand={b} withName size={22} />)}
                </div>
              </div>

              <span className="mt-5 inline-flex font-mono text-xs text-[color:var(--green)] hover:underline">
                {lang === "fr" ? "Voir le format en détail →" : "See format in detail →"}
              </span>
            </Link>
          );
        })}
      </div>

      <section className="mt-20">
        <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)] mb-3">
          {lang === "fr" ? "// Écosystème" : "// Ecosystem"}
        </div>
        <h2 className="t-h2">
          {lang === "fr" ? "Ces entreprises utilisent ces formats au quotidien." : "These companies use these formats every day."}
        </h2>
        <p className="mt-3 text-[color:var(--text-2)] max-w-2xl">
          {lang === "fr"
            ? "UMC sert d'intermédiaire neutre entre tous les acteurs de l'écosystème — sans verrouillage matériel ni dépendance à un éditeur."
            : "UMC is the neutral middleman between every actor of the ecosystem — no hardware lock-in, no vendor dependency."}
        </p>
        <div className="mt-8"><CompanyDropdown /></div>
      </section>
    </PageStub>
  );
}