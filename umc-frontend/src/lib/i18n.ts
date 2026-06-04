import { useTheme } from "./theme";

const DICT = {
  nav: {
    formats:     { fr: "Formats",      en: "Formats" },
    workshop:    { fr: "Atelier",      en: "Workshop" },
    hub:         { fr: "Hub",          en: "Hub" },
    benchmarks:  { fr: "Benchmarks",   en: "Benchmarks" },
    blog:        { fr: "Blog",         en: "Blog" },
    docs:        { fr: "Docs",         en: "Docs" },
    pricing:     { fr: "Tarifs",       en: "Pricing" },
    try:         { fr: "Convertir",    en: "Convert" },
  },
  hero: {
    badge:    { fr: "Service en ligne · zéro installation · 31 formats", en: "Online service · zero install · 31 formats" },
    titleA:   { fr: "Le ",              en: "The " },
    titleHi:  { fr: "standard universel", en: "universal standard" },
    titleB:   { fr: " de conversion des modèles IA.", en: " for converting AI models." },
    sub:      {
      fr: "Importez votre modèle, choisissez le format cible, téléchargez le résultat — sans rien installer. UMC est au modèle IA ce que ffmpeg est à la vidéo.",
      en: "Drop in your model, pick a target format, download the result — no install required. UMC is to AI models what ffmpeg is to video.",
    },
    cta:      { fr: "Convertir maintenant", en: "Convert now" },
    secondary:{ fr: "Voir l'univers",      en: "See the universe" },
  },
  sections: {
    trusted:    { fr: "Utilisé par les équipes qui livrent.", en: "Used by teams that ship." },
    universe:   { fr: "L'univers de conversion UMC", en: "The UMC conversion universe" },
    universeSub:{ fr: "31 formats, 280+ chemins de conversion certifiés, un seul service en ligne.",
                  en: "31 formats, 280+ certified conversion paths, one online service." },
    problem:    { fr: "Les problèmes qu'UMC résout", en: "The problems UMC solves" },
    why:        { fr: "Pourquoi UMC ?", en: "Why UMC?" },
    companies:  { fr: "Ces entreprises utilisent ces formats au quotidien",
                  en: "These companies use these formats every day" },
    flux:       { fr: "Le flux mondial de conversion IA", en: "Global AI conversion flow" },
    perf:       { fr: "Performances réelles, mesurées", en: "Real, measured performance" },
    cta:        { fr: "Convertissez votre premier modèle en 4 secondes.",
                  en: "Convert your first model in 4 seconds." },
  },
};

type Leaf = { fr: string; en: string };
type Tree = { [k: string]: Leaf | Tree };

function get(obj: Tree, path: string): Leaf | undefined {
  return path.split(".").reduce<any>((o, k) => (o ? o[k] : undefined), obj);
}

export function useT() {
  const { lang } = useTheme();
  return (key: string) => {
    const leaf = get(DICT as Tree, key);
    return leaf ? leaf[lang] : key;
  };
}