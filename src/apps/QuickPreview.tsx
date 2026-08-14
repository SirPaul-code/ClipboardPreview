import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { CSSProperties } from 'react';
import type { QuickPreviewPayload } from '../types';
export function QuickPreview(){
  const [payload,setPayload]=useState<QuickPreviewPayload|null>(null);
  useEffect(()=>{let off:undefined|(()=>void); listen<QuickPreviewPayload>('clipboard://quick-preview',e=>setPayload(e.payload)).then(fn=>off=fn); return()=>off?.();},[]);
  const p=payload; const item=p?.item;
  return <main className="overlay-shell quick" style={p?{'--overlay-opacity':p.appearance.overlayOpacity,'--overlay-radius':`${p.appearance.cornerRadius}px`,'--overlay-font':`${p.settings.fontSize}px`} as CSSProperties:undefined}>
    <div className="overlay-accent"/><div className="preview-content">
      <div className="overlay-kicker">{item ? item.type.toUpperCase() : 'CLIPBOARD'}</div>
      <div className={`preview-text ${item?.type==='code'?'mono':''}`} style={{whiteSpace:p?.settings.textWrapping?'pre-wrap':'nowrap',WebkitLineClamp:p?.settings.maxLines}}>{item?.content?.slice(0,p?.settings.maxCharacters) || 'Clipboard is empty'}</div>
      {p?.settings.showCharacterCount && item && <div className="preview-count">{item.metadata.characterCount} characters</div>}
    </div>
  </main>;
}
