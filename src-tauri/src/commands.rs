use std::collections::HashSet;
use tauri::{AppHandle,Manager};
use tauri_plugin_autostart::ManagerExt;
use crate::{clipboard,models::{AppSettings,InteractionMode,PlatformStatus},overlays,permissions,selection,settings_store,shortcuts,state::AppState};
#[tauri::command]pub fn get_settings(app:AppHandle)->AppSettings{app.state::<AppState>().settings.read().clone()}
#[tauri::command]pub fn get_history(app:AppHandle)->Vec<crate::models::ClipboardItem>{app.state::<AppState>().history.lock().items()}
fn unique_shortcuts(s:&AppSettings)->bool{let xs=[&s.shortcuts.quick_preview,&s.shortcuts.history_selector,&s.shortcuts.open_settings,&s.shortcuts.pause_monitoring];let set:HashSet<_>=xs.iter().map(|x|x.to_lowercase()).collect();set.len()==xs.len()}
#[tauri::command]pub fn save_settings(app:AppHandle,settings:AppSettings)->Result<AppSettings,String>{
    let next=settings.normalized(); if !unique_shortcuts(&next){return Err("Each action needs a unique shortcut".into())}
    let old=app.state::<AppState>().settings.read().clone(); *app.state::<AppState>().settings.write()=next.clone();
    if let Err(e)=shortcuts::register_all(&app){*app.state::<AppState>().settings.write()=old.clone();let _=shortcuts::register_all(&app);return Err(e)}
    let startup_result=if next.general.launch_at_startup{app.autolaunch().enable()}else{app.autolaunch().disable()};
    if let Err(e)=startup_result{*app.state::<AppState>().settings.write()=old;let _=shortcuts::register_all(&app);return Err(e.to_string())}
    if let Some(tray)=app.tray_by_id("main-tray"){tray.set_visible(next.general.show_tray_icon).map_err(|e|e.to_string())?;}
    app.state::<AppState>().history.lock().truncate(next.history.max_items); settings_store::save_settings(&app,&next)?;
    let items=app.state::<AppState>().history.lock().items(); settings_store::save_history(&app,next.history.persist_history,&items)?; Ok(next)
}
#[tauri::command]pub fn clear_history(app:AppHandle)->Result<(),String>{app.state::<AppState>().history.lock().clear();settings_store::clear_history_file(&app);Ok(())}
#[tauri::command]pub fn select_history_item(app:AppHandle,id:String)->Result<(),String>{let state=app.state::<AppState>();let settings=state.settings.read().clone();let item=state.history.lock().find(&id).ok_or("History item not found")?;clipboard::write_text(&app,&item.content)?;if settings.history.move_selected_to_top{state.history.lock().promote(&id);let items=state.history.lock().items();settings_store::save_history(&app,settings.history.persist_history,&items)?;}overlays::hide_history(&app);Ok(())}
#[tauri::command]pub fn navigate_selection(app:AppHandle,delta:i32)->Result<(),String>{selection::navigate(&app,delta)}
#[tauri::command]pub fn accept_selection(app:AppHandle)->Result<(),String>{selection::accept(&app)}
#[tauri::command]pub fn cancel_selection(app:AppHandle)->Result<(),String>{selection::cancel(&app);Ok(())}
#[tauri::command]pub fn show_history(app:AppHandle)->Result<(),String>{overlays::begin(&app,InteractionMode::Sticky)}
#[tauri::command]pub fn open_settings(app:AppHandle)->Result<(),String>{overlays::open_settings(&app)}
#[tauri::command]pub fn toggle_monitoring(app:AppHandle)->Result<bool,String>{let value={let mut s=app.state::<AppState>().settings.write();s.general.monitoring_paused=!s.general.monitoring_paused;settings_store::save_settings(&app,&s)?;s.general.monitoring_paused};Ok(value)}
#[tauri::command]pub fn platform_status(app:AppHandle)->PlatformStatus{let mac=cfg!(target_os="macos");let access=permissions::accessibility_granted();PlatformStatus{os:std::env::consts::OS.into(),accessibility_required:mac,accessibility_granted:access,hold_release_available:!mac||access,global_wheel_available:!mac||access,version:app.package_info().version.to_string()}}
#[tauri::command]pub fn complete_first_run(app:AppHandle)->Result<AppSettings,String>{let mut s=app.state::<AppState>().settings.write();s.first_run_completed=true;settings_store::save_settings(&app,&s)?;Ok(s.clone())}
#[tauri::command]pub fn reset_settings(app:AppHandle)->Result<AppSettings,String>{let defaults=AppSettings::default().normalized();*app.state::<AppState>().settings.write()=defaults.clone();shortcuts::register_all(&app)?;app.autolaunch().disable().map_err(|e|e.to_string())?;if let Some(tray)=app.tray_by_id("main-tray"){let _=tray.set_visible(defaults.general.show_tray_icon);}settings_store::save_settings(&app,&defaults)?;settings_store::save_history(&app,false,&[])?;Ok(defaults)}
#[tauri::command]pub fn quit_app(app:AppHandle){app.exit(0)}
