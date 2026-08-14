import { useEffect, useState } from 'react';
import type { CSSProperties } from 'react';
import { listen } from '@tauri-apps/api/event';
import { backend } from '../lib/tauri';
import type { HistoryPayload } from '../types';
import { ClipboardCard } from '../components/ClipboardCard';
export function HistoryOverlay(){
  const [payload,setPayload]=useState<HistoryPayload|null>(null);
  useEffect(()=>{const offs:Array<()=>void>=[]; Promise.all(['clipboard://history-show','clipboard://history-selection'].map(name=>listen<HistoryPayload>(name,e=>setPayload(e.payload)))).then(xs=>offs.push(...xs)); return()=>offs.forEach(fn=>fn());},[]);
  useEffect(()=>{const onKey=(e:KeyboardEvent)=>{if(!payload||payload.interactionMode!=='sticky')return; if(['ArrowDown','j','J'].includes(e.key)){e.preventDefault();void backend.navigate(1);} else if(['ArrowUp','k','K'].includes(e.key)){e.preventDefault();void backend.navigate(-1);} else if(e.key==='Enter'){e.preventDefault();void backend.accept();} else if(e.key==='Escape'){e.preventDefault();void backend.cancel();}}; window.addEventListener('keydown',onKey); return()=>window.removeEventListener('keydown',onKey);},[payload]);
  useEffect(()=>{const wheel=(e:WheelEvent)=>{if(!payload||payload.interactionMode!=='sticky')return; e.preventDefault();void backend.navigate(e.deltaY>0?1:-1);}; window.addEventListener('wheel',wheel,{passive:false});return()=>window.removeEventListener('wheel',wheel);},[payload]);
  return <main className="overlay-shell history" style={payload?{'--overlay-opacity':payload.appearance.overlayOpacity,'--overlay-radius':`${payload.appearance.cornerRadius}px`} as CSSProperties:undefined}>
    <header className="history-header"><div><div className="brand-small">Clipboard Preview</div><div className="history-sub">Recent clipboard</div></div><span className="count-badge">{payload?.totalItems ?? 0}</span></header>
    <div className="history-list">{payload?.items.length ? payload.items.map((item,i)=><ClipboardCard key={item.id} item={item} selected={i===payload.selectedIndex} onClick={()=>void backend.selectItem(item.id)}/>) : <div className="empty-state"><strong>No clipboard history yet</strong><span>Copy something to get started.</span></div>}</div>
    <footer className="history-footer">{payload?.interactionMode==='sticky'?'↑ ↓ navigate · Enter select · Esc cancel':'Scroll to select · release shortcut to use'}</footer>
  </main>;
}
