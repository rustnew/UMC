import type { ReactNode } from "react";

/** Tiny SVG explainer schemas used inside blog articles. */
export function QuantizationSchema() {
  return (
    <Frame caption="Quantification : FP32 → INT4. Chaque poids compressé, qualité quasi intacte.">
      <svg viewBox="0 0 400 140" className="w-full">
        <Row y={30} label="FP32" color="#4D9EFF" boxes={16} fills={16} />
        <Row y={70} label="FP16" color="#00FF94" boxes={16} fills={12} />
        <Row y={110} label="INT4" color="#FF7E2D" boxes={16} fills={4} />
        <text x="380" y="34" textAnchor="end" fontSize="10" className="font-mono" fill="var(--text-3)">4 octets/poids</text>
        <text x="380" y="74" textAnchor="end" fontSize="10" className="font-mono" fill="var(--text-3)">2 octets/poids</text>
        <text x="380" y="114" textAnchor="end" fontSize="10" className="font-mono" fill="var(--text-3)">0.5 octet/poids</text>
      </svg>
    </Frame>
  );
}

export function DistillationSchema() {
  return (
    <Frame caption="Distillation : un grand modèle 'enseigne' un petit modèle, qui hérite de son savoir.">
      <svg viewBox="0 0 400 160" className="w-full">
        <circle cx="80" cy="80" r="42" fill="var(--bg-3)" stroke="#4D9EFF" />
        <text x="80" y="76" textAnchor="middle" fontSize="11" className="font-mono" fill="var(--text-1)">Teacher</text>
        <text x="80" y="92" textAnchor="middle" fontSize="9" className="font-mono" fill="var(--text-3)">70B</text>
        <circle cx="320" cy="80" r="26" fill="var(--bg-3)" stroke="var(--green)" />
        <text x="320" y="78" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--text-1)">Student</text>
        <text x="320" y="92" textAnchor="middle" fontSize="9" className="font-mono" fill="var(--text-3)">7B</text>
        <path d="M 130 80 C 200 30, 230 30, 290 70" fill="none" stroke="var(--green)" strokeWidth="1.5" strokeDasharray="4 3" />
        <text x="210" y="40" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--green)">soft labels</text>
        <path d="M 290 90 C 240 130, 200 130, 130 90" fill="none" stroke="var(--text-3)" strokeOpacity="0.4" strokeWidth="1" />
      </svg>
    </Frame>
  );
}

export function PruningSchema() {
  return (
    <Frame caption="Élagage : on supprime les connexions inutiles. Le réseau reste précis, plus rapide.">
      <svg viewBox="0 0 400 160" className="w-full">
        {[40, 80, 120].map((x, i) =>
          [30, 70, 110, 150].map((y, j) => (
            <circle key={`l${i}-${j}`} cx={x} cy={y} r="4" fill="var(--green)" />
          ))
        )}
        {[200, 240, 280].map((x, i) =>
          [30, 70, 110, 150].map((y, j) => {
            const dropped = (i + j) % 3 === 0;
            return <circle key={`r${i}-${j}`} cx={x} cy={y} r="4" fill={dropped ? "var(--bg-4)" : "var(--green)"} />;
          })
        )}
        <text x="80" y="18" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--text-3)">Dense</text>
        <text x="240" y="18" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--text-3)">Pruned (−40%)</text>
      </svg>
    </Frame>
  );
}

export function FfmpegSchema() {
  return (
    <Frame caption="Comme ffmpeg unifie la vidéo, UMC unifie les modèles IA.">
      <svg viewBox="0 0 400 160" className="w-full">
        {["mp4", "mov", "mkv", "avi"].map((f, i) => (
          <g key={f}><rect x={20} y={20 + i * 28} width="60" height="20" rx="3" fill="var(--bg-3)" stroke="var(--text-3)" />
            <text x="50" y="34" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--text-2)">{f}</text>
            <path d={`M 80 ${30 + i * 28} L 175 80`} stroke="var(--text-3)" strokeOpacity="0.4" /></g>
        ))}
        <rect x="170" y="65" width="60" height="30" rx="4" fill="var(--bg-3)" stroke="var(--green)" />
        <text x="200" y="84" textAnchor="middle" fontSize="11" className="font-mono" fill="var(--green)">UMC</text>
        {["gguf", "onnx", "coreml", "tflite"].map((f, i) => (
          <g key={f}><rect x={320} y={20 + i * 28} width="60" height="20" rx="3" fill="var(--bg-3)" stroke="var(--text-3)" />
            <text x="350" y="34" textAnchor="middle" fontSize="10" className="font-mono" fill="var(--text-2)">{f}</text>
            <path d={`M 230 80 L 320 ${30 + i * 28}`} stroke="var(--green)" strokeOpacity="0.4" /></g>
        ))}
      </svg>
    </Frame>
  );
}

function Frame({ children, caption }: { children: ReactNode; caption: string }) {
  return (
    <figure className="my-8 p-5 rounded-xl border border-[color:var(--border)] bg-[color:var(--bg-2)]">
      {children}
      <figcaption className="mt-3 text-center font-mono text-xs text-[color:var(--text-3)]">{caption}</figcaption>
    </figure>
  );
}

function Row({ y, label, color, boxes, fills }: { y: number; label: string; color: string; boxes: number; fills: number }) {
  return (
    <g>
      <text x="10" y={y + 12} fontSize="11" className="font-mono" fill="var(--text-2)">{label}</text>
      {Array.from({ length: boxes }).map((_, i) => (
        <rect key={i} x={60 + i * 18} y={y} width="14" height="14" rx="2"
          fill={i < fills ? color : "var(--bg-3)"} fillOpacity={i < fills ? 1 : 0.5} />
      ))}
    </g>
  );
}