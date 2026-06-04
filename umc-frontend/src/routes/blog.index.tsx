import { createFileRoute, Link } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";
import { POSTS } from "@/lib/posts";
import { useTheme } from "@/lib/theme";

export const Route = createFileRoute("/blog/")({ component: Page });

function Page() {
  const { lang } = useTheme();
  return (
    <PageStub
      eyebrow={lang === "fr" ? "Blog technique" : "Technical blog"}
      title={lang === "fr" ? "Une référence sur la conversion de modèles IA." : "A reference on AI model conversion."}
      description={lang === "fr"
        ? "Articles longs avec schémas, comparaisons rigoureuses, benchmarks reproductibles. Pour ingénieurs ML qui veulent comprendre."
        : "Long-form articles with diagrams, rigorous comparisons, reproducible benchmarks. For ML engineers who want to understand."}
    >
      <div className="grid md:grid-cols-2 gap-6">
        {POSTS.map((p, i) => {
          const palettes = [
            ["#00FF94","#38E1FF"],
            ["#B66BFF","#FF4FD8"],
            ["#FFC93C","#FF7E2D"],
            ["#38E1FF","#B66BFF"],
            ["#B8FF59","#00FF94"],
            ["#FF4FD8","#FFC93C"],
          ];
          const [c1, c2] = palettes[i % palettes.length];
          return (
            <Link key={p.slug} to="/blog/$slug" params={{ slug: p.slug }}
              className="group rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] hover:border-[color:var(--text-3)] transition overflow-hidden">
              {/* aurora cover */}
              <div
                className="h-32 relative overflow-hidden"
                style={{ background: `linear-gradient(120deg, ${c1}, ${c2})` }}
              >
                <div className="absolute inset-0 opacity-30 mix-blend-overlay"
                  style={{ backgroundImage: "radial-gradient(circle at 20% 30%, rgba(255,255,255,0.5), transparent 40%), radial-gradient(circle at 80% 70%, rgba(0,0,0,0.4), transparent 50%)" }} />
                <div className="absolute bottom-2 left-3 right-3 flex items-center justify-between">
                  <span className="px-2 py-0.5 rounded-full bg-black/40 backdrop-blur text-white font-mono text-[10px] uppercase tracking-widest">{p.tag}</span>
                  <span className="font-mono text-[10px] text-white/80">{p.read} min de lecture</span>
                </div>
              </div>
              <div className="p-6">
                <h3 className="t-h2 !text-xl group-hover:text-[color:var(--green)] transition">{p.title[lang]}</h3>
                <p className="mt-2 text-sm text-[color:var(--text-2)] leading-relaxed">{p.summary[lang]}</p>
                <span className="mt-4 inline-flex items-center gap-1 font-mono text-xs text-[color:var(--green)] opacity-0 group-hover:opacity-100 transition">
                  Lire l'article →
                </span>
              </div>
            </Link>
          );
        })}
      </div>
    </PageStub>
  );
}