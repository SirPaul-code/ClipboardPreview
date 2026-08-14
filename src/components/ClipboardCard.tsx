import type { ClipboardItem } from '../types';
const labels = { text:'Text', url:'URL', code:'Code', multiline:'Multiline' } as const;
export function ClipboardCard({ item, selected=false, onClick }: { item:ClipboardItem; selected?:boolean; onClick?:()=>void }) {
  return <button type="button" className={`clip-card ${selected?'selected':''}`} onClick={onClick}>
    <div className="clip-main"><span className={`type-pill ${item.type}`}>{labels[item.type]}</span><div className={`clip-text ${item.type==='code'?'mono':''}`}>{item.preview || 'Empty text'}</div></div>
    <span className="clip-meta">{item.metadata.characterCount} ch</span>
  </button>;
}
