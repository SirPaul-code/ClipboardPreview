import { useEffect, useState } from 'react';
import { prettyShortcut, shortcutFromEvent } from '../lib/shortcuts';
export function ShortcutRecorder({ value, onChange }: { value:string; onChange:(v:string)=>void }) {
  const [recording,setRecording]=useState(false);
  useEffect(()=>{
    if(!recording) return;
    const handler=(event:KeyboardEvent)=>{ event.preventDefault(); event.stopPropagation(); if(event.key==='Escape'){setRecording(false);return;} const next=shortcutFromEvent(event); if(next){onChange(next);setRecording(false);} };
    window.addEventListener('keydown',handler,true); return()=>window.removeEventListener('keydown',handler,true);
  },[recording,onChange]);
  return <button className={`shortcut-recorder ${recording?'recording':''}`} onClick={()=>setRecording(true)}>{recording?'Press shortcut…':prettyShortcut(value)}</button>;
}
