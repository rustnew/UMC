import { useTheme, t } from "@/lib/theme";
import { Moon, Sun, Palette, UserCircle2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Logo } from "./Logo";
import { useAuth } from "@/lib/auth";

export function Nav() {
  const { theme, setTheme, lang, setLang } = useTheme();
  const { session } = useAuth();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const items = [
    { to: "/app", fr: "Atelier", en: "Workshop" },
    { to: "/formats", fr: "Formats", en: "Formats" },
    { to: "/hub", fr: "Hub", en: "Hub" },
    { to: "/personas", fr: "Pour qui", en: "For who" },
    { to: "/companies", fr: "Entreprises", en: "Companies" },
    { to: "/use-cases", fr: "Cas d'usage", en: "Use cases" },
    { to: "/benchmarks", fr: "Performances", en: "Performance" },
    { to: "/blog", fr: "Blog", en: "Blog" },
    { to: "/pricing", fr: "Tarifs", en: "Pricing" },
  ];

  const cycleTheme = () => {
    const order: Array<"dark" | "light" | "gruvbox"> = ["dark", "gruvbox", "light"];
    const i = order.indexOf(theme);
    setTheme(order[(i + 1) % order.length]);
  };

  const ThemeIcon = theme === "light" ? Sun : theme === "gruvbox" ? Palette : Moon;

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled ? "backdrop-blur-xl bg-[color:var(--bg-1)]/75 border-b border-[color:var(--border)]" : ""
      }`}
    >
      <nav className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <a href="/" className="flex items-center gap-2 group">
          <Logo size={26} className="transition-transform group-hover:rotate-[60deg] duration-700" />
          <span className="font-mono font-semibold tracking-tight text-[color:var(--text-1)]">UMC</span>
        </a>

        <div className="hidden md:flex items-center gap-7 text-sm">
          {items.map((it) => (
            <a
              key={it.to}
              href={it.to}
              className="text-[color:var(--text-2)] hover:text-[color:var(--text-1)] transition-colors"
            >
              {t({ fr: it.fr, en: it.en }, lang)}
            </a>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => setLang(lang === "fr" ? "en" : "fr")}
            className="font-mono text-xs uppercase px-2.5 py-1.5 rounded-md border border-[color:var(--border)] text-[color:var(--text-2)] hover:text-[color:var(--text-1)] hover:border-[color:var(--text-3)] transition"
            aria-label="Switch language"
          >
            {lang}
          </button>
          <button
            onClick={cycleTheme}
            className="p-2 rounded-md border border-[color:var(--border)] text-[color:var(--text-2)] hover:text-[color:var(--text-1)] hover:border-[color:var(--text-3)] transition"
            aria-label="Switch theme"
          >
            <ThemeIcon size={15} />
          </button>
          {session ? (
            <a href="/account" className="ml-1 hidden sm:inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-[color:var(--border)] hover:border-[color:var(--green)] text-sm transition">
              <UserCircle2 size={14} /> {t({ fr: "Compte", en: "Account" }, lang)}
            </a>
          ) : (
            <a href="/login" className="ml-1 hidden sm:inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-[color:var(--border)] hover:border-[color:var(--text-3)] text-sm transition text-[color:var(--text-2)]">
              {t({ fr: "Connexion", en: "Login" }, lang)}
            </a>
          )}
          <a
            href={session ? "/app" : "/signup"}
            className="ml-0.5 hidden sm:inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-md text-[color:var(--bg-0)] font-medium text-sm hover:brightness-110 transition shadow-[0_8px_24px_-8px_rgba(0,255,148,0.55)]"
            style={{ backgroundImage: "var(--gradient-brand)" }}
          >
            {session ? t({ fr: "Convertir", en: "Convert" }, lang) : t({ fr: "Créer un compte", en: "Sign up" }, lang)} →
          </a>
        </div>
      </nav>
    </header>
  );
}