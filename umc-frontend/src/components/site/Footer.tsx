import { Logo } from "./Logo";
import { useTheme, t } from "@/lib/theme";

export function Footer() {
  const { lang } = useTheme();
  return (
    <footer className="relative border-t border-[color:var(--border)] mt-32 overflow-hidden">
      {/* aurora wash on top */}
      <div className="absolute inset-x-0 top-0 h-px aurora-bg opacity-80" />
      <div className="absolute inset-0 pointer-events-none opacity-[0.06] aurora-bg" />
      <div className="relative max-w-7xl mx-auto px-6 py-20">
        <div className="grid lg:grid-cols-[1.4fr_1fr_1fr_1fr_1fr] gap-10 mb-14">
          <div>
            <div className="flex items-center gap-2.5 mb-4">
              <Logo size={26} />
            </div>
            <p className="text-sm text-[color:var(--text-2)] leading-relaxed max-w-sm">
              {t({ fr: "Le standard universel de conversion des modèles IA.", en: "The universal standard for AI model conversion." }, lang)}
              <br />
              <span className="text-[color:var(--text-3)]">
                {t({ fr: "Une usine de conversion en ligne — 32 formats, 280+ chemins certifiés, zéro installation.", en: "An online conversion factory — 32 formats, 280+ certified paths, zero install." }, lang)}
              </span>
            </p>
            <div className="mt-6 flex items-center gap-2">
              {["var(--green)","var(--cyan)","var(--violet)","var(--magenta)","var(--amber)","var(--orange)"].map((c, i) => (
                <span key={i} className="w-2.5 h-2.5 rounded-full" style={{ background: c, boxShadow: `0 0 12px ${c}` }} />
              ))}
            </div>
          </div>
          <Col title={t({ fr: "Produit", en: "Product" }, lang)} links={[
            [t({ fr: "Atelier", en: "Workshop" }, lang), "/app"],
            ["Hub", "/hub"],
            ["Formats", "/formats"],
            [t({ fr: "Tarifs", en: "Pricing" }, lang), "/pricing"],
            [t({ fr: "Garanties", en: "Guarantees" }, lang), "/guarantees"],
          ]} />
          <Col title={t({ fr: "Écosystème", en: "Ecosystem" }, lang)} links={[
            [t({ fr: "Entreprises", en: "Companies" }, lang), "/companies"],
            [t({ fr: "Pour qui", en: "For who" }, lang), "/personas"],
            [t({ fr: "Cas d'usage", en: "Use cases" }, lang), "/use-cases"],
            [t({ fr: "Performances", en: "Benchmarks" }, lang), "/benchmarks"],
          ]} />
          <Col title={t({ fr: "Ressources", en: "Resources" }, lang)} links={[
            ["Blog", "/blog"],
            ["Docs", "/docs"],
            ["API", "/docs"],
            [t({ fr: "Connexion", en: "Login" }, lang), "/login"],
          ]} />
          <Col title={t({ fr: "Légal", en: "Legal" }, lang)} links={[
            [t({ fr: "Cookies", en: "Cookies" }, lang), "/legal/cookies"],
            [t({ fr: "Confidentialité", en: "Privacy" }, lang), "/legal/privacy"],
            [t({ fr: "Conditions", en: "Terms" }, lang), "/legal/terms"],
          ]} />
        </div>

        {/* big aurora wordmark */}
        <div className="border-t border-[color:var(--border)] pt-10 pb-2">
          <div className="select-none text-center">
            <div className="text-[18vw] lg:text-[14vw] leading-[0.85] font-display font-medium tracking-tighter aurora-bg bg-clip-text text-transparent">
              UMC
            </div>
          </div>
        </div>

        <div className="mt-6 flex flex-col sm:flex-row items-center justify-between gap-3 font-mono text-xs text-[color:var(--text-3)]">
          <span>© 2025 UMC — Universal Model Converter</span>
          <span className="flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-[color:var(--green)] animate-pulse" />
            Service en ligne · δ &lt; 1e-6 · Certificat ed25519
          </span>
        </div>
      </div>
    </footer>
  );
}

function Col({ title, links }: { title: string; links: Array<[string, string]> }) {
  return (
    <div>
      <div className="font-mono text-xs uppercase tracking-widest text-gradient-brand mb-3">{title}</div>
      <ul className="space-y-2">
        {links.map(([label, to]) => (
          <li key={to}>
            <a href={to} className="text-sm text-[color:var(--text-2)] hover:text-[color:var(--text-1)] hover:translate-x-0.5 inline-block transition">
              {label}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
}