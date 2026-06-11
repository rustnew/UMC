import { createFileRoute, useNavigate, Link } from "@tanstack/react-router";
import { useEffect, useState, type ReactNode } from "react";
import { toast } from "sonner";
import { Nav } from "@/components/site/Nav";
import { Footer } from "@/components/site/Footer";
import { useAuth } from "@/lib/auth";
import { LogOut, Mail, Building2, UserCircle2, ShieldCheck } from "lucide-react";

export const Route = createFileRoute("/account")({
  component: AccountPage,
  head: () => ({ meta: [{ title: "Mon compte — UMC" }] }),
});

function AccountPage() {
  const { session, user, loading: authLoading, signOut } = useAuth();
  const navigate = useNavigate();
  const [displayName, setDisplayName] = useState("");
  const [organization, setOrganization] = useState("");
  const [persona, setPersona] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!authLoading && !session) navigate({ to: "/login", replace: true });
  }, [authLoading, session, navigate]);

  useEffect(() => {
    if (!user) return;
    setDisplayName(user.display_name ?? "");
  }, [user]);

  if (authLoading || !user) {
    return (
      <div className="min-h-screen grid place-items-center bg-[color:var(--bg-1)] text-[color:var(--text-2)] font-mono text-sm">
        Chargement…
      </div>
    );
  }

  const save = async () => {
    setSaving(true);
    await new Promise((r) => setTimeout(r, 300));
    setSaving(false);
    toast.success("Préférences enregistrées localement");
  };

  const onSignOut = async () => {
    await signOut();
    navigate({ to: "/" });
  };

  return (
    <div className="min-h-screen bg-[color:var(--bg-1)] text-[color:var(--text-1)]">
      <Nav />
      <main className="pt-32 pb-20 px-6">
        <div className="max-w-3xl mx-auto">
          <div className="font-mono text-xs uppercase tracking-widest text-[color:var(--green)]">// Mon compte</div>
          <h1 className="t-h1 mt-3">Bonjour {displayName || user.email}</h1>
          <p className="mt-3 text-[color:var(--text-2)]">Gérez votre profil, vos conversions et vos préférences.</p>

          <div className="mt-10 grid md:grid-cols-3 gap-3">
            <Stat icon={<Mail size={16} />} label="Email" value={user.email ?? "—"} />
            <Stat icon={<ShieldCheck size={16} />} label="Statut" value="Vérifié" accent="var(--green)" />
            <Stat icon={<UserCircle2 size={16} />} label="Compte" value={user.plan ?? "Free"} />
          </div>

          <section className="mt-10 rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7">
            <h2 className="text-lg font-semibold">Profil</h2>
            <p className="text-sm text-[color:var(--text-3)] mt-1">Ces informations nous aident à mieux adapter UMC à votre usage.</p>

            <div className="mt-6 space-y-3">
              <Field label="Nom complet"><input value={displayName} onChange={e=>setDisplayName(e.target.value)} className="input" /></Field>
              <Field label="Organisation"><input value={organization} onChange={e=>setOrganization(e.target.value)} className="input" /></Field>
              <Field label="Profil utilisateur">
                <select value={persona} onChange={e=>setPersona(e.target.value)} className="input">
                  <option value="">—</option>
                  <option value="mlops">MLOps / Déploiement IA</option>
                  <option value="research">Recherche ML</option>
                  <option value="enterprise">Industrie régulée</option>
                  <option value="mobile">Mobile / Edge</option>
                  <option value="startup">Startup IA</option>
                  <option value="other">Autre</option>
                </select>
              </Field>

              <div className="flex flex-wrap gap-3 pt-2">
                <button onClick={save} disabled={saving}
                  className="px-5 py-2.5 rounded-lg text-[color:var(--bg-0)] font-semibold text-sm hover:brightness-110 transition disabled:opacity-50"
                  style={{ backgroundImage: "var(--gradient-brand)" }}>
                  {saving ? "..." : "Enregistrer"}
                </button>
                <button onClick={onSignOut}
                  className="inline-flex items-center gap-2 px-4 py-2.5 rounded-lg border border-[color:var(--border)] hover:border-[color:var(--magenta)] hover:text-[color:var(--magenta)] text-sm transition">
                  <LogOut size={14} /> Se déconnecter
                </button>
              </div>
            </div>
          </section>

          <section className="mt-6 rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-7">
            <h2 className="text-lg font-semibold flex items-center gap-2"><Building2 size={18} /> Accès rapide</h2>
            <div className="mt-4 grid sm:grid-cols-3 gap-3">
              <Link to="/app" className="rounded-lg border border-[color:var(--border)] p-4 hover:border-[color:var(--green)] transition">
                <div className="font-medium">Atelier</div>
                <div className="text-xs text-[color:var(--text-3)] mt-1">Convertir un modèle</div>
              </Link>
              <Link to="/hub" className="rounded-lg border border-[color:var(--border)] p-4 hover:border-[color:var(--cyan)] transition">
                <div className="font-medium">Hub</div>
                <div className="text-xs text-[color:var(--text-3)] mt-1">Modèles disponibles</div>
              </Link>
              <Link to="/pricing" className="rounded-lg border border-[color:var(--border)] p-4 hover:border-[color:var(--violet)] transition">
                <div className="font-medium">Tarifs</div>
                <div className="text-xs text-[color:var(--text-3)] mt-1">Passer Pro / Enterprise</div>
              </Link>
            </div>
          </section>
        </div>
      </main>
      <Footer />
      <style>{`
        .input { width: 100%; padding: 0.625rem 0.875rem; border-radius: 0.5rem;
          background: var(--bg-1); border: 1px solid var(--border); font-size: 0.875rem;
          color: var(--text-1); transition: border-color .15s; }
        .input:focus { outline: none; border-color: var(--green); }
      `}</style>
    </div>
  );
}

function Stat({ icon, label, value, accent }: { icon: ReactNode; label: string; value: string; accent?: string }) {
  return (
    <div className="rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-4">
      <div className="flex items-center gap-2 text-[color:var(--text-3)] text-xs font-mono uppercase tracking-widest">
        <span style={{ color: accent ?? "var(--text-2)" }}>{icon}</span>{label}
      </div>
      <div className="mt-2 text-sm font-medium truncate" style={accent ? { color: accent } : undefined}>{value}</div>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="block text-xs font-mono uppercase tracking-widest text-[color:var(--text-3)] mb-1.5">{label}</span>
      {children}
    </label>
  );
}