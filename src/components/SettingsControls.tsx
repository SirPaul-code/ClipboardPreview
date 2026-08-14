import type { ReactNode } from 'react';
export function Section({ title, description, children }: { title: string; description?: string; children: ReactNode }) {
  return <section className="settings-section"><div className="section-heading"><h2>{title}</h2>{description && <p>{description}</p>}</div><div className="settings-card">{children}</div></section>;
}
export function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return <div className="settings-row"><div><div className="row-label">{label}</div>{hint && <div className="row-hint">{hint}</div>}</div><div className="row-control">{children}</div></div>;
}
export function Toggle({ checked, onChange, disabled=false }: { checked: boolean; onChange: (v:boolean)=>void; disabled?: boolean }) {
  return <button type="button" disabled={disabled} aria-pressed={checked} className={`toggle ${checked?'on':''}`} onClick={() => onChange(!checked)}><span /></button>;
}
export function NumberField({ value, min, max, step=1, suffix, onChange }: { value:number; min:number; max:number; step?:number; suffix?:string; onChange:(v:number)=>void }) {
  return <label className="number-field"><input type="number" value={value} min={min} max={max} step={step} onChange={e=>onChange(Math.min(max,Math.max(min,Number(e.target.value))))}/>{suffix && <span>{suffix}</span>}</label>;
}
