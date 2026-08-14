use std::{sync::atomic::Ordering,thread,time::Duration};
use tauri::{AppHandle,Emitter,LogicalPosition,LogicalSize,Manager,Position,Size};
use crate::{models::{HistoryOverlayPayload,InteractionMode,OverlayPosition,QuickPreviewPayload},state::AppState};

fn position(app:&AppHandle,label:&str,width:u32,height:u32,pos:&OverlayPosition)->Result<(),String>{
    let w=app.get_webview_window(label).ok_or_else(||format!("Window {label} missing"))?;
    let cursor=app.cursor_position().map_err(|e|e.to_string())?;
    let monitor=app.monitor_from_point(cursor.x,cursor.y).map_err(|e|e.to_string())?.or_else(||app.primary_monitor().ok().flatten()).ok_or("No monitor available")?;
    let scale=monitor.scale_factor(); let area=monitor.work_area();
    let ax=area.position.x as f64/scale; let ay=area.position.y as f64/scale; let aw=area.size.width as f64/scale; let ah=area.size.height as f64/scale;
    let (x,y)=match pos{OverlayPosition::Cursor=>(cursor.x/scale+18.0,cursor.y/scale+20.0),OverlayPosition::ScreenCenter=>(ax+(aw-width as f64)/2.0,ay+(ah-height as f64)/2.0),OverlayPosition::TopCenter=>(ax+(aw-width as f64)/2.0,ay+48.0),OverlayPosition::BottomCenter=>(ax+(aw-width as f64)/2.0,ay+ah-height as f64-48.0)};
    let x=x.clamp(ax+8.0,(ax+aw-width as f64-8.0).max(ax+8.0)); let y=y.clamp(ay+8.0,(ay+ah-height as f64-8.0).max(ay+8.0));
    w.set_size(Size::Logical(LogicalSize::new(width as f64,height as f64))).map_err(|e|e.to_string())?;
    w.set_position(Position::Logical(LogicalPosition::new(x,y))).map_err(|e|e.to_string())?; Ok(())
}

pub fn show_quick(app:&AppHandle)->Result<(),String>{
    let state=app.state::<AppState>(); let s=state.settings.read().clone();
    let payload=QuickPreviewPayload{item:state.history.lock().get(0),settings:s.preview.clone(),appearance:s.appearance.clone()};
    let height=((s.preview.max_lines.min(6) as u32*s.preview.font_size*3)/2+48).clamp(84,240);
    position(app,"quick-preview",s.preview.width,height,&s.preview.position)?;
    let w=app.get_webview_window("quick-preview").ok_or("Quick preview missing")?;
    w.set_focusable(false).map_err(|e|e.to_string())?;
    app.emit_to("quick-preview","clipboard://quick-preview",payload).map_err(|e|e.to_string())?;
    w.show().map_err(|e|e.to_string())?;
    let generation=state.preview_generation.fetch_add(1,Ordering::SeqCst)+1; let delay=s.preview.auto_hide_ms; let handle=app.clone();
    thread::spawn(move||{thread::sleep(Duration::from_millis(delay));let state=handle.state::<AppState>();if state.preview_generation.load(Ordering::SeqCst)==generation{if let Some(w)=handle.get_webview_window("quick-preview"){let _=w.hide();}}}); Ok(())
}

fn payload(app:&AppHandle)->HistoryOverlayPayload{
    let state=app.state::<AppState>(); let s=state.settings.read().clone(); let all=state.history.lock().items();
    let absolute=state.selector.lock().selected_index.min(all.len().saturating_sub(1)); let visible=s.history.visible_items.max(1);
    let start=absolute.saturating_sub(visible.saturating_sub(1)/2).min(all.len().saturating_sub(visible)); let end=(start+visible).min(all.len());
    HistoryOverlayPayload{items:all[start..end].to_vec(),selected_index:absolute.saturating_sub(start),interaction_mode:s.history.interaction_mode,appearance:s.appearance,total_items:all.len()}
}
pub fn show_history(app:&AppHandle,focus:bool)->Result<(),String>{let s=app.state::<AppState>().settings.read().clone();let h=(106+s.history.visible_items as u32*52).clamp(260,700);position(app,"history-overlay",520,h,&s.preview.position)?;let w=app.get_webview_window("history-overlay").ok_or("History overlay missing")?;w.set_focusable(focus).map_err(|e|e.to_string())?;app.emit_to("history-overlay","clipboard://history-show",payload(app)).map_err(|e|e.to_string())?;w.show().map_err(|e|e.to_string())?;if focus{w.set_focus().map_err(|e|e.to_string())?}Ok(())}
pub fn emit_selection(app:&AppHandle)->Result<(),String>{app.emit_to("history-overlay","clipboard://history-selection",payload(app)).map_err(|e|e.to_string())}
pub fn begin(app:&AppHandle,mode:InteractionMode)->Result<(),String>{{let state=app.state::<AppState>();let mut x=state.selector.lock();x.active=true;x.selected_index=0;}show_history(app,matches!(mode,InteractionMode::Sticky))}
pub fn hide_history(app:&AppHandle){if let Some(w)=app.get_webview_window("history-overlay"){let _=w.hide();let _=w.set_focusable(false);}}
pub fn open_settings(app:&AppHandle)->Result<(),String>{let w=app.get_webview_window("settings").ok_or("Settings window missing")?;w.show().map_err(|e|e.to_string())?;let _=w.unminimize();w.set_focus().map_err(|e|e.to_string())}
