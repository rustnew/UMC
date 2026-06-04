import { useEffect, useState } from "react";

/**
 * Animated conversion graph — pure SVG, no D3.
 * Nodes = formats, edges = conversion paths, packets travel along edges.
 */
const NODES = [
  { id: "pytorch", label: "PyTorch", x: 140, y: 60, color: "#EE4C2C" },
  { id: "safetensors", label: "SafeTensors", x: 60, y: 180, color: "#FFD21E" },
  { id: "onnx", label: "ONNX", x: 260, y: 220, color: "#005CED" },
  { id: "gguf", label: "GGUF", x: 420, y: 90, color: "#00FF94" },
  { id: "coreml", label: "CoreML", x: 460, y: 240, color: "#A2AAAD" },
  { id: "tflite", label: "TFLite", x: 150, y: 320, color: "#FF6F00" },
  { id: "tensorrt", label: "TRT", x: 380, y: 360, color: "#76B900" },
];

const EDGES: Array<[string, string]> = [
  ["pytorch", "safetensors"],
  ["pytorch", "onnx"],
  ["safetensors", "gguf"],
  ["onnx", "coreml"],
  ["onnx", "tensorrt"],
  ["safetensors", "tflite"],
  ["onnx", "gguf"],
  ["pytorch", "coreml"],
];

const byId = (id: string) => NODES.find((n) => n.id === id)!;

export function ConversionGraph() {
  const [activeIdx, setActiveIdx] = useState(0);

  useEffect(() => {
    const id = setInterval(() => setActiveIdx((i) => (i + 1) % NODES.length), 1600);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="relative w-full aspect-[5/4] rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] overflow-hidden">
      {/* grid bg */}
      <svg className="absolute inset-0 w-full h-full opacity-[0.07]" aria-hidden="true">
        <defs>
          <pattern id="grid" width="32" height="32" patternUnits="userSpaceOnUse">
            <path d="M 32 0 L 0 0 0 32" fill="none" stroke="currentColor" strokeWidth="0.5" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />
      </svg>

      <svg viewBox="0 0 520 420" className="relative w-full h-full">
        <defs>
          <radialGradient id="halo">
            <stop offset="0%" stopColor="#00FF94" stopOpacity="0.45" />
            <stop offset="100%" stopColor="#00FF94" stopOpacity="0" />
          </radialGradient>
          <linearGradient id="edgeGrad" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" stopColor="#00FF94" stopOpacity="0.05" />
            <stop offset="50%" stopColor="#00FF94" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#00FF94" stopOpacity="0.05" />
          </linearGradient>
        </defs>

        {/* edges */}
        {EDGES.map(([a, b], i) => {
          const na = byId(a), nb = byId(b);
          return (
            <g key={i}>
              <line
                x1={na.x} y1={na.y} x2={nb.x} y2={nb.y}
                stroke="url(#edgeGrad)" strokeWidth="1.2"
              />
              {/* packet */}
              <circle r="3" fill="#00FF94">
                <animateMotion
                  dur={`${2.4 + (i % 3) * 0.6}s`}
                  repeatCount="indefinite"
                  path={`M ${na.x} ${na.y} L ${nb.x} ${nb.y}`}
                  begin={`${i * 0.4}s`}
                />
                <animate attributeName="opacity" values="0;1;1;0" dur={`${2.4 + (i % 3) * 0.6}s`} repeatCount="indefinite" />
              </circle>
            </g>
          );
        })}

        {/* nodes */}
        {NODES.map((n, i) => {
          const active = i === activeIdx;
          return (
            <g key={n.id} transform={`translate(${n.x} ${n.y})`}>
              {active && <circle r="32" fill="url(#halo)" />}
              <circle
                r="20"
                fill="var(--bg-1)"
                stroke={active ? "#00FF94" : "#2A2F38"}
                strokeWidth={active ? 1.5 : 1}
              />
              <circle r="6" fill={n.color} opacity={active ? 1 : 0.7} />
              <text
                y="36" textAnchor="middle"
                className="font-mono"
                fontSize="10"
                fill={active ? "#F0F2F5" : "#9AA3B0"}
              >
                {n.label}
              </text>
            </g>
          );
        })}
      </svg>

      {/* HUD */}
      <div className="absolute top-3 left-3 flex items-center gap-2 font-mono text-[10px] text-[color:var(--text-2)]">
        <span className="w-1.5 h-1.5 rounded-full bg-[color:var(--green)] animate-pulse" />
        UMC.CONVERT — live
      </div>
      <div className="absolute top-3 right-3 font-mono text-[10px] text-[color:var(--text-3)]">
        7 / 31 formats
      </div>
      <div className="absolute bottom-3 left-3 right-3 flex items-center justify-between font-mono text-[10px] text-[color:var(--text-3)]">
        <span>tensors: 291 · checked ✓</span>
        <span className="text-[color:var(--green)]">δ &lt; 1e-6</span>
      </div>
    </div>
  );
}