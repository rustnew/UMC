import { useMemo, useState } from "react";

export function ROICalculator() {
  const [engineers, setEngineers] = useState(8);
  const [hours, setHours] = useState(12);
  const [cost, setCost] = useState(85);

  const savings = useMemo(() => {
    const weeksPerYear = 48;
    const before = engineers * hours * cost * weeksPerYear;
    const after = before * 0.04; // ~96% reduction
    return Math.round(before - after);
  }, [engineers, hours, cost]);

  return (
    <div className="grid lg:grid-cols-[1fr_1fr] gap-8 rounded-2xl border border-[color:var(--border)] bg-[color:var(--bg-2)] p-8">
      <div className="space-y-6">
        <Slider label="Ingénieurs ML" value={engineers} min={1} max={50} onChange={setEngineers} suffix="" />
        <Slider label="Heures / semaine sur les conversions" value={hours} min={1} max={40} onChange={setHours} suffix="h" />
        <Slider label="Coût horaire" value={cost} min={30} max={250} onChange={setCost} suffix="€" />
      </div>

      <div className="flex flex-col justify-between">
        <div>
          <div className="font-mono text-xs text-[color:var(--text-3)] uppercase tracking-widest mb-2">
            Économie annuelle estimée
          </div>
          <div className="t-metric">
            {savings.toLocaleString("fr-FR")} €
          </div>
          <div className="font-mono text-xs text-[color:var(--text-2)] mt-2">
            soit ~96% de temps libéré
          </div>
        </div>

        <div className="mt-8 grid grid-cols-2 gap-px bg-[color:var(--border)] rounded-lg overflow-hidden text-sm">
          <div className="bg-[color:var(--bg-3)] p-4">
            <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)]">Avant</div>
            <div className="font-mono mt-1 text-[color:var(--red)]">45 min · 64 Go RAM</div>
          </div>
          <div className="bg-[color:var(--bg-3)] p-4">
            <div className="font-mono text-[10px] uppercase text-[color:var(--text-3)]">Avec UMC</div>
            <div className="font-mono mt-1 text-[color:var(--green)]">4.2 s · 800 Mo RAM</div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Slider({
  label, value, min, max, onChange, suffix,
}: { label: string; value: number; min: number; max: number; onChange: (n: number) => void; suffix: string }) {
  return (
    <label className="block">
      <div className="flex items-baseline justify-between mb-2">
        <span className="text-sm text-[color:var(--text-2)]">{label}</span>
        <span className="font-mono text-sm text-[color:var(--green)]">{value}{suffix}</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-[color:var(--green)] h-1"
      />
    </label>
  );
}