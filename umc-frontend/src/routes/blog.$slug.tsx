import { createFileRoute, Link, notFound } from "@tanstack/react-router";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { POSTS } from "@/lib/posts";
import { QuantizationSchema, DistillationSchema, PruningSchema, FfmpegSchema } from "@/components/site/Schema";
import { useTheme } from "@/lib/theme";
import { ArrowLeft } from "lucide-react";

export const Route = createFileRoute("/blog/$slug")({
  component: Article,
  loader: ({ params }) => {
    const post = POSTS.find((p) => p.slug === params.slug);
    if (!post) throw notFound();
    return { post };
  },
  notFoundComponent: () => (
    <div className="min-h-screen grid place-items-center text-[color:var(--text-2)]">Article introuvable.</div>
  ),
});

function Article() {
  const { post } = Route.useLoaderData();
  const { lang } = useTheme();
  const paragraphs = post.body[lang];
  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-32 pb-20 px-6">
        <article className="max-w-3xl mx-auto">
          <Link to="/blog" className="inline-flex items-center gap-1.5 font-mono text-xs text-[color:var(--text-3)] hover:text-[color:var(--text-1)]">
            <ArrowLeft size={12} /> {lang === "fr" ? "Tous les articles" : "All articles"}
          </Link>
          <div className="mt-6 font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">{post.tag} · {post.read} min</div>
          <h1 className="t-h1 mt-3">{post.title[lang]}</h1>
          <p className="mt-4 text-lg text-[color:var(--text-2)]">{post.summary[lang]}</p>

          <div className="mt-10 space-y-6 text-[color:var(--text-2)] leading-relaxed">
            {paragraphs.map((p: string, i: number) => (
              <div key={i}>
                <p>{p}</p>
                {/* interleave schemas */}
                {post.schemas[i] === "quantization" && <QuantizationSchema />}
                {post.schemas[i] === "distillation" && <DistillationSchema />}
                {post.schemas[i] === "pruning" && <PruningSchema />}
                {post.schemas[i] === "ffmpeg" && <FfmpegSchema />}
              </div>
            ))}
            {/* trailing schema if not yet rendered */}
            {post.schemas.length > 0 && post.schemas.length > paragraphs.length && (
              post.schemas.slice(paragraphs.length).map((s: string, i: number) => {
                if (s === "quantization") return <QuantizationSchema key={`t${i}`} />;
                if (s === "distillation") return <DistillationSchema key={`t${i}`} />;
                if (s === "pruning") return <PruningSchema key={`t${i}`} />;
                return <FfmpegSchema key={`t${i}`} />;
              })
            )}
            {/* always show first schema if not yet rendered inline */}
            {post.schemas[0] && paragraphs.length < 2 && null}
          </div>

          <div className="mt-14 pt-8 border-t border-[color:var(--border)] flex items-center justify-between">
            <Link to="/blog" className="font-mono text-xs text-[color:var(--text-3)] hover:text-[color:var(--text-1)]">← {lang === "fr" ? "Retour" : "Back"}</Link>
            <a href="/app" className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[color:var(--green)] text-[color:var(--bg-0)] font-medium text-sm">
              {lang === "fr" ? "Essayer UMC" : "Try UMC"} →
            </a>
          </div>
        </article>
      </main>
      <Footer />
    </div>
  );
}