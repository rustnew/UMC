import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

type Theme = "dark" | "light" | "gruvbox";
type Lang = "fr" | "en";

interface Ctx {
  theme: Theme;
  setTheme: (t: Theme) => void;
  lang: Lang;
  setLang: (l: Lang) => void;
}

const ThemeCtx = createContext<Ctx | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>("dark");
  const [lang, setLangState] = useState<Lang>("fr");

  useEffect(() => {
    const t = (localStorage.getItem("umc-theme") as Theme) || "dark";
    const l = (localStorage.getItem("umc-lang") as Lang) || "fr";
    setThemeState(t);
    setLangState(l);
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const setTheme = (t: Theme) => {
    setThemeState(t);
    localStorage.setItem("umc-theme", t);
  };
  const setLang = (l: Lang) => {
    setLangState(l);
    localStorage.setItem("umc-lang", l);
  };

  return (
    <ThemeCtx.Provider value={{ theme, setTheme, lang, setLang }}>
      {children}
    </ThemeCtx.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeCtx);
  if (!ctx) throw new Error("useTheme must be inside ThemeProvider");
  return ctx;
}

/** tiny dict translator */
export function t<T extends Record<Lang, string>>(d: T, lang: Lang) {
  return d[lang];
}