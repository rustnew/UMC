import { createFileRoute } from "@tanstack/react-router";
import { PageStub } from "@/components/site/PageStub";

export const Route = createFileRoute("/docs")({ component: Page });

function Page() {
  return (
    <PageStub
      eyebrow="Documentation"
      title="Démarrer avec UMC en 60 secondes."
      description="Installation, premier convertissement, options avancées. Tout est ici."
    >
      <div className="space-y-8">
        <Block title="Installation" code={`curl -fsSL https://umc.dev/install.sh | sh`} />
        <Block title="Première conversion" code={`umc convert model.safetensors --to gguf --quant Q4_K_M
# → model.gguf  (4.2s · 800 Mo RAM)
# → model.umc.cert  (signature ed25519)`} />
        <Block title="Inspection" code={`umc inspect model.gguf
# architecture: llama
# params: 8.03B
# tensors: 291 (checked ✓)
# checksum: 9f3a...c102`} />
        <Block title="Diff entre formats" code={`umc diff model.safetensors model.gguf --metric max-divergence
# δ_max = 8.7e-3  (within Q4_K_M tolerance)`} />
      </div>
    </PageStub>
  );
}

function Block({ title, code }: { title: string; code: string }) {
  return (
    <div>
      <h2 className="t-h2 !text-xl mb-3">{title}</h2>
      <pre className="font-mono text-sm bg-[color:var(--bg-0)] p-4 rounded-lg border border-[color:var(--border)] text-[color:var(--text-2)] overflow-x-auto whitespace-pre">
{code}
      </pre>
    </div>
  );
}