import { BRANDS, BrandMark, type BrandKey } from "@/lib/brands";
import { Link } from "@tanstack/react-router";

const ROW: BrandKey[] = [
  "meta", "openai", "google", "microsoft", "apple", "nvidia", "mistral",
  "huggingface", "anthropic", "stability", "intel", "amd", "qualcomm",
  "tencent", "alibaba", "baidu", "samsung", "tesla", "bmw", "airbus",
  "spotify", "shopify", "snapchat", "ibm", "amazon", "deepseek",
];

export function Ticker() {
  const items = [...ROW, ...ROW];
  return (
    <div className="relative overflow-hidden border-y border-[color:var(--border)] bg-[color:var(--bg-2)]/40 py-7">
      <div className="font-mono text-[10px] uppercase tracking-[0.25em] text-[color:var(--text-3)] text-center mb-4">
        Adopté par les équipes IA chez
      </div>
      <div className="flex gap-14 animate-ticker whitespace-nowrap will-change-transform">
        {items.map((b, i) => (
          <Link key={i} to="/companies/$slug" params={{ slug: b }}
            className="flex items-center gap-2.5 shrink-0 opacity-70 hover:opacity-100 transition-opacity duration-300">
            <BrandMark brand={b} size={26} />
            <span className="font-mono text-sm text-[color:var(--text-2)]">{BRANDS[b].name}</span>
          </Link>
        ))}
      </div>
      <div className="pointer-events-none absolute inset-y-0 left-0 w-40 bg-gradient-to-r from-[color:var(--bg-1)] via-[color:var(--bg-1)]/80 to-transparent" />
      <div className="pointer-events-none absolute inset-y-0 right-0 w-40 bg-gradient-to-l from-[color:var(--bg-1)] via-[color:var(--bg-1)]/80 to-transparent" />
    </div>
  );
}