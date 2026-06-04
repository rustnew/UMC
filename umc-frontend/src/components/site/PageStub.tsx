import { Nav } from "./Nav";
import { Footer } from "./Footer";

export function PageStub({
  eyebrow, title, description, children,
}: { eyebrow: string; title: string; description: string; children?: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-5xl mx-auto">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">// {eyebrow}</div>
          <h1 className="t-h1 mt-3">{title}</h1>
          <p className="mt-4 text-lg text-[color:var(--text-2)] max-w-2xl">{description}</p>
          <div className="mt-12">{children}</div>
        </div>
      </main>
      <Footer />
    </div>
  );
}