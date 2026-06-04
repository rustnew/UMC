import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { BrandMark, BRANDS } from "@/lib/brands";
import { COMPANIES_USING_FORMATS, FORMATS } from "@/lib/formats";
import { useTheme } from "@/lib/theme";

export function CompanyDropdown() {
  const [open, setOpen] = useState(false);
  const { lang } = useTheme();
  const visible = open ? COMPANIES_USING_FORMATS : COMPANIES_USING_FORMATS.slice(0, 6);

  return (
    <div>
      <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-px bg-[color:var(--border)] rounded-xl overflow-hidden">
        {visible.map((c) => (
          <div key={c.brand} className="bg-[color:var(--bg-2)] p-5 hover:bg-[color:var(--bg-3)] transition">
            <div className="flex items-center justify-between">
              <BrandMark brand={c.brand} withName size={28} />
            </div>
            <p className="mt-3 text-sm text-[color:var(--text-2)] leading-relaxed">{c.blurb[lang]}</p>
            <div className="mt-3 flex flex-wrap gap-1.5">
              {c.formats.map((slug) => {
                const f = FORMATS.find((x) => x.slug === slug);
                if (!f) return null;
                return (
                  <span key={slug} className="font-mono text-[10px] px-1.5 py-0.5 rounded border" style={{ borderColor: f.color + "55", color: f.color }}>
                    {f.name}
                  </span>
                );
              })}
            </div>
          </div>
        ))}
      </div>
      <button
        onClick={() => setOpen((v) => !v)}
        className="mt-5 inline-flex items-center gap-2 px-4 py-2 rounded-md border border-[color:var(--border)] hover:border-[color:var(--text-3)] font-mono text-xs text-[color:var(--text-2)]"
      >
        {open
          ? (lang === "fr" ? "Réduire" : "Show less")
          : (lang === "fr" ? `Voir les ${COMPANIES_USING_FORMATS.length} entreprises` : `See all ${COMPANIES_USING_FORMATS.length} companies`)}
        {open ? <ChevronUp size={13} /> : <ChevronDown size={13} />}
      </button>
      {!open && (
        <span className="ml-3 font-mono text-xs text-[color:var(--text-3)]">
          {Object.keys(BRANDS).length}+ entreprises servies
        </span>
      )}
    </div>
  );
}