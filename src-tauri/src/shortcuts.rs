use std::str::FromStr;
use tauri::{AppHandle,Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt,Shortcut,ShortcutState};
use crate::{models::InteractionMode,overlays,selection,state::AppState};
fn parse(s:&str)->Option<Shortcut>{Shortcut::from_str(s).ok()}
fn action(app:&AppHandle,shortcut:&Shortcut,event_state:ShortcutState){
    let state=app.state::<AppState>();
    let settings=state.settings.read().clone();
    let q=parse(&settings.shortcuts.quick_preview); let h=parse(&settings.shortcuts.history_selector); let o=parse(&settings.shortcuts.open_settings); let p=parse(&settings.shortcuts.pause_monitoring);
    if q.as_ref()==Some(shortcut)&&matches!(event_state,ShortcutState::Pressed){let _=overlays::show_quick(app);}
    else if h.as_ref()==Some(shortcut){match event_state{ShortcutState::Pressed=>{if !state.selector.lock().active{let _=overlays::begin(app,settings.history.interaction_mode.clone());}},ShortcutState::Released=>{if matches!(settings.history.interaction_mode,InteractionMode::HoldRelease){let _=selection::accept(app);}}}}
    else if o.as_ref()==Some(shortcut)&&matches!(event_state,ShortcutState::Pressed){let _=overlays::open_settings(app);}
    else if p.as_ref()==Some(shortcut)&&matches!(event_state,ShortcutState::Pressed){let mut s=state.settings.write();s.general.monitoring_paused=!s.general.monitoring_paused;let _=crate::settings_store::save_settings(app,&s);}
}
pub fn plugin()->tauri::plugin::TauriPlugin<tauri::Wry>{tauri_plugin_global_shortcut::Builder::new().with_handler(|app,shortcut,event|action(app,shortcut,event.state())).build()}
pub fn register_all(app:&AppHandle)->Result<(),String>{let s=app.state::<AppState>().settings.read().clone();app.global_shortcut().unregister_all().map_err(|e|e.to_string())?;for shortcut in [&s.shortcuts.quick_preview,&s.shortcuts.history_selector,&s.shortcuts.open_settings,&s.shortcuts.pause_monitoring]{app.global_shortcut().register(shortcut.as_str()).map_err(|e|format!("Could not register {shortcut}: {e}"))?;}Ok(())}
