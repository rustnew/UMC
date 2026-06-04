import { useEffect, useState } from "react";
import { useTheme, t } from "@/lib/theme";
import { Cookie, X } from "lucide-react";

const KEY = "umc.cookies.v1";
type Choice = "all" | "essential" | null;

export function CookieConsent() {
  const { lang } = useTheme();
  const [choice, setChoice] = useState<Choice>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    const saved = localStorage.getItem(KEY) as Choice;
    setChoice(saved ?? null);
  }, []);

  const decide = (c: Exclude<Choice, null>) => {
    localStorage.setItem(KEY, c);
    setChoice(c);
  };

  if (!mounted || choice) return null;

  return (
    <div className="fixed inset-x-3 bottom-3 z-[70] sm:left-auto sm:right-5 sm:bottom-5 sm:max-w-md animate-float-up">
      <div className="rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-1)]/95 backdrop-blur-xl shadow-2xl p-5">
        <div className="flex items-start gap-3">
          <div className="shrink-0 w-9 h-9 rounded-lg grid place-items-center bg-[color:var(--bg-2)] text-[color:var(--green)]">
            <Cookie size={16} />
          </div>
          <div className="min-w-0 flex-1">
            <div className="text-sm font-medium text-[color:var(--text-1)]">
              {t({ fr: "Cookies & confidentialité", en: "Cookies & privacy" }, lang)}
            </div>
            <p className="mt-1 text-xs text-[color:var(--text-3)] leading-relaxed">
              {t({
                fr: "Nous utilisons des cookies essentiels au fonctionnement du site et, avec votre accord, des cookies de mesure d'audience anonymes pour améliorer UMC.",
                en: "We use essential cookies to run the site and, with your consent, anonymous analytics cookies to improve UMC.",
              }, lang)}{" "}
              <a href="/legal/cookies" className="text-[color:var(--green)] hover:underline">
                {t({ fr: "En savoir plus", en: "Learn more" }, lang)}
              </a>
            </p>
            <div className="mt-4 flex flex-wrap gap-2">
              <button
                onClick={() => decide("all")}
                className="px-3 py-1.5 rounded-md text-xs font-medium bg-[color:var(--green)] text-[color:var(--bg-0)] hover:brightness-110 transition"
              >
                {t({ fr: "Tout accepter", en: "Accept all" }, lang)}
              </button>
              <button
                onClick={() => decide("essential")}
                className="px-3 py-1.5 rounded-md text-xs border border-[color:var(--border)] text-[color:var(--text-2)] hover:text-[color:var(--text-1)] hover:border-[color:var(--text-3)] transition"
              >
                {t({ fr: "Essentiels uniquement", en: "Essential only" }, lang)}
              </button>
              <a
                href="/legal/cookies"
                className="px-3 py-1.5 rounded-md text-xs text-[color:var(--text-3)] hover:text-[color:var(--text-1)] transition"
              >
                {t({ fr: "Personnaliser", en: "Customize" }, lang)}
              </a>
            </div>
          </div>
          <button
            onClick={() => decide("essential")}
            aria-label="close"
            className="shrink-0 p-1 rounded-md text-[color:var(--text-3)] hover:text-[color:var(--text-1)] hover:bg-[color:var(--bg-3)] transition"
          >
            <X size={14} />
          </button>
        </div>
      </div>
    </div>
  );
}