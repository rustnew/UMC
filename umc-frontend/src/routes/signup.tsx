import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useState, useEffect, type FormEvent } from "react";
import { toast } from "sonner";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { Logo } from "@/components/site/Logo";
import { auth as authApi } from "@/integrations/api/client";
import { useAuth, useAuthSuccess } from "@/lib/auth";

export const Route = createFileRoute("/signup")({
  component: SignupPage,
  head: () => ({ meta: [{ title: "Créer un compte — UMC" }] }),
});

const PERSONAS = [
  { v: "mlops", l: "MLOps / Déploiement IA" },
  { v: "research", l: "Recherche ML" },
  { v: "enterprise", l: "Industrie régulée (santé, finance, défense)" },
  { v: "mobile", l: "Développement mobile / edge" },
  { v: "startup", l: "Startup IA" },
  { v: "other", l: "Autre" },
];

function SignupPage() {
  const navigate = useNavigate();
  const { session } = useAuth();
  const handleAuthSuccess = useAuthSuccess();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (session) navigate({ to: "/account", replace: true });
  }, [session, navigate]);

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const res = await authApi.register(email, password, displayName || undefined);
      handleAuthSuccess(res.user, res.access_token, res.refresh_token);
      toast.success("Compte créé ! Bienvenue.");
      navigate({ to: "/account" });
    } catch (err: unknown) {
      toast.error(err instanceof Error ? err.message : "Erreur d'inscription");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-md mx-auto">
          <div className="text-center mb-8">
            <Logo size={44} className="mx-auto" />
            <h1 className="t-h2 mt-4">Créer votre compte UMC</h1>
            <p className="mt-2 text-sm text-[color:var(--text-3)]">Conversion gratuite jusqu'à 4 Go · certificat ed25519 inclus</p>
          </div>

          <div className="rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7">
            <form onSubmit={onSubmit} className="space-y-3">
              <input placeholder="Nom complet (optionnel)" value={displayName} onChange={e=>setDisplayName(e.target.value)}
                className="w-full px-3.5 py-2.5 rounded-lg bg-[color:var(--bg-1)] border border-[color:var(--border)] text-sm focus:outline-none focus:border-[color:var(--green)] transition" />
              <input required type="email" placeholder="Email professionnel" value={email} onChange={e=>setEmail(e.target.value)}
                className="w-full px-3.5 py-2.5 rounded-lg bg-[color:var(--bg-1)] border border-[color:var(--border)] text-sm focus:outline-none focus:border-[color:var(--green)] transition" />
              <input required type="password" placeholder="Mot de passe (8+ caractères)" minLength={8} value={password} onChange={e=>setPassword(e.target.value)}
                className="w-full px-3.5 py-2.5 rounded-lg bg-[color:var(--bg-1)] border border-[color:var(--border)] text-sm focus:outline-none focus:border-[color:var(--green)] transition" />
              <button type="submit" disabled={loading}
                className="w-full py-2.5 rounded-lg text-[color:var(--bg-0)] font-semibold text-sm hover:brightness-110 transition disabled:opacity-50"
                style={{ backgroundImage: "var(--gradient-brand)" }}>
                {loading ? "..." : "Créer mon compte"}
              </button>
            </form>
          </div>

          <p className="mt-6 text-center text-sm text-[color:var(--text-3)]">
            Déjà un compte ? <Link to="/login" className="text-[color:var(--green)] hover:underline">Se connecter</Link>
          </p>
        </div>
      </main>
      <Footer />
    </div>
  );
}

function GoogleIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24">
      <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
      <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
      <path fill="#FBBC05" d="M5.84 14.1c-.22-.66-.35-1.36-.35-2.1s.13-1.44.35-2.1V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l3.66-2.84z"/>
      <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84C6.71 7.31 9.14 5.38 12 5.38z"/>
    </svg>
  );
}