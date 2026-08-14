use std::{thread,time::Duration};
use arboard::Clipboard;
use tauri::{AppHandle,Emitter,Manager};
use crate::{history::hash_text,settings_store,state::AppState};

pub fn start(app:AppHandle){
    thread::spawn(move||{
        let mut clipboard:Option<Clipboard>=None;
        loop{
            let (paused,interval,max,persist)={let s=app.state::<AppState>().settings.read().clone();(s.general.monitoring_paused,s.advanced.clipboard_poll_interval_ms,s.history.max_items,s.history.persist_history)};
            if !paused{
                if clipboard.is_none(){clipboard=Clipboard::new().ok()}
                if let Some(cb)=clipboard.as_mut(){
                    if let Ok(text)=cb.get_text(){
                        let hash=hash_text(&text);
                        let state=app.state::<AppState>();
                        let internal={let mut token=state.internal_write_hash.lock();if token.as_deref()==Some(&hash){token.take();true}else{false}};
                        if !internal{
                            let changed=state.history.lock().add_text(text,max);
                            if changed{
                                let items=state.history.lock().items();
                                if let Err(e)=settings_store::save_history(&app,persist,&items){log::warn!("Could not persist clipboard history: {e}")}
                                let _=app.emit("clipboard://history-changed",());
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(interval));
        }
    });
}

pub fn write_text(app:&AppHandle,text:&str)->Result<(),String>{
    let hash=hash_text(text);
    *app.state::<AppState>().internal_write_hash.lock()=Some(hash);
    let mut cb=Clipboard::new().map_err(|e|format!("Clipboard unavailable: {e}"))?;
    if let Err(e)=cb.set_text(text.to_owned()){
        *app.state::<AppState>().internal_write_hash.lock()=None;
        return Err(format!("Unable to write clipboard: {e}"));
    }
    Ok(())
}
