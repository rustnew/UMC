import { useEffect, useMemo, useState } from "react";
import { FORMATS } from "@/lib/formats";

/**
 * Animated radial map of the AI format universe.
 * Center hub = UMC. Each orbiting node = one format with its brand color.
 * Packets travel from random formats to the hub and back to random targets.
 */
export function ConversionUniverse() {
  const [tick, setTick] = useState(0);
  const [active, setActive] = useState<Set<number>>(new Set());

  // Random illumination — every 700ms light up 2-3 random formats
  useEffect(() => {
    const id = setInterval(() => {
      setTick((t) => t + 1);
      const next = new Set<number>();
      const total = Math.min(FORMATS.length, 16);
      const count = 2 + Math.floor(Math.random() * 2);
      for (let i = 0; i < count; i++) next.add(Math.floor(Math.random() * total));
      setActive(next);
    }, 750);
    return () => clearInterval(id);
  }, []);

  const cx = 260, cy = 230;
  const orbits = useMemo(() => ([
    { r: 105, items: FORMATS.slice(0, 6),  speed: 80 },
    { r: 175, items: FORMATS.slice(6, 16), speed: 60 },
  ]), []);

  // Pre-compute flat node list with positions
  const nodes = useMemo(() => {
    const out: Array<{ idx: number; x: number; y: number; color: string; name: string; slug: string }> = [];
    let idx = 0;
    orbits.forEach((o, oi) => {
      o.items.forEach((f, i) => {
        const a = (i / o.items.length) * Math.PI * 2 + oi * 0.35;
        out.push({ idx: idx++, x: cx + Math.cos(a) * o.r, y: cy + Math.sin(a) * o.r, color: f.color, name: f.name, slug: f.slug });
      });
    });
    return out;
  }, [orbits]);

  return (
    <div className="relative w-full aspect-[5/4] rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] overflow-hidden shadow-[var(--shadow-glow)]">
      {/* aurora wash */}
      <div className="absolute inset-0 opacity-[0.18] aurora-bg pointer-events-none" />
      <svg className="absolute inset-0 w-full h-full opacity-[0.07]" aria-hidden="true">
        <defs>
          <pattern id="uni-grid" width="28" height="28" patternUnits="userSpaceOnUse">
            <path d="M 28 0 L 0 0 0 28" fill="none" stroke="currentColor" strokeWidth="0.5" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#uni-grid)" />
      </svg>

      <svg viewBox="0 0 520 460" className="relative w-full h-full">
        <defs>
          <radialGradient id="uni-halo">
            <stop offset="0%" stopColor="var(--green)" stopOpacity="0.75" />
            <stop offset="50%" stopColor="var(--cyan)" stopOpacity="0.35" />
            <stop offset="100%" stopColor="var(--green)" stopOpacity="0" />
          </radialGradient>
          <linearGradient id="uni-ring" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%"  stopColor="var(--green)" />
            <stop offset="50%" stopColor="var(--cyan)" />
            <stop offset="100%" stopColor="var(--violet)" />
          </linearGradient>
        </defs>

        {/* orbit rings */}
        {orbits.map((o) => (
          <circle key={o.r} cx={cx} cy={cy} r={o.r} fill="none" stroke="url(#uni-ring)" strokeOpacity="0.22" strokeDasharray="3 6" />
        ))}

        {/* convergence beams — drawn from every node toward the hub, intensity boosted when active */}
        {nodes.map((n) => {
          const isOn = active.has(n.idx);
          return (
            <line
              key={`beam-${n.idx}`}
              x1={n.x} y1={n.y} x2={cx} y2={cy}
              stroke={n.color}
              strokeWidth={isOn ? 1.6 : 0.6}
              strokeOpacity={isOn ? 0.85 : 0.12}
              style={{ transition: "stroke-opacity 600ms ease, stroke-width 600ms ease" }}
            />
          );
        })}

        {/* center hub halo */}
        <circle cx={cx} cy={cy} r="60" fill="url(#uni-halo)">
          <animate attributeName="r" values="55;72;55" dur="3.4s" repeatCount="indefinite" />
        </circle>
        <circle cx={cx} cy={cy} r="34" fill="var(--bg-1)" stroke="url(#uni-ring)" strokeWidth="2" />
        <text x={cx} y={cy + 5} textAnchor="middle" fontSize="14" className="font-mono" fill="var(--green)" fontWeight="600">UMC</text>

        {/* convergence packets — flowing toward the hub from every active node */}
        {nodes.map((n) => {
          const isOn = active.has(n.idx);
          if (!isOn) return null;
          const pathIn = `M ${n.x} ${n.y} L ${cx} ${cy}`;
          return (
            <g key={`pkt-${n.idx}-${tick}`}>
              <circle r="3.2" fill={n.color}>
                <animateMotion dur="0.9s" repeatCount="1" path={pathIn} fill="freeze" />
                <animate attributeName="opacity" values="0;1;1;0" dur="0.9s" repeatCount="1" />
              </circle>
            </g>
          );
        })}

        {/* nodes */}
        {nodes.map((n) => {
          const isOn = active.has(n.idx);
          return (
            <g key={n.slug} transform={`translate(${n.x} ${n.y})`}>
              {isOn && (
                <>
                  <circle r="28" fill={n.color} fillOpacity="0.18">
                    <animate attributeName="r" values="18;32;18" dur="1.2s" repeatCount="1" />
                    <animate attributeName="fill-opacity" values="0.35;0;0.35" dur="1.2s" repeatCount="1" />
                  </circle>
                  <circle r="20" fill={n.color} fillOpacity="0.10" />
                </>
              )}
              <circle r="16" fill="var(--bg-1)" stroke={n.color} strokeWidth={isOn ? 2.4 : 1.2}
                style={{ transition: "stroke-width 400ms ease" }} />
              <text textAnchor="middle" y="4" fontSize="8.5" className="font-mono" fontWeight="600"
                fill={isOn ? n.color : "var(--text-2)"}
                style={{ transition: "fill 400ms ease" }}>
                {n.name.length > 6 ? n.name.slice(0, 5) : n.name}
              </text>
            </g>
          );
        })}
      </svg>

      <div className="absolute top-3 left-3 flex items-center gap-2 font-mono text-[10px] text-[color:var(--text-2)]">
        <span className="w-1.5 h-1.5 rounded-full bg-[color:var(--green)] animate-pulse" />
        UMC · LIVE
      </div>
      <div className="absolute top-3 right-3 font-mono text-[10px] text-[color:var(--text-3)]">
        {FORMATS.length}+ / 31 formats
      </div>
      <div className="absolute bottom-3 left-3 right-3 flex items-center justify-between font-mono text-[10px] text-[color:var(--text-3)]">
        <span>280+ chemins · convergence permanente</span>
        <span className="text-[color:var(--green)]">δ &lt; 1e-6</span>
      </div>
    </div>
  );
}