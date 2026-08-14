use std::{fs,path::PathBuf};
use tauri::{AppHandle,Manager};
use crate::models::{AppSettings,ClipboardItem};
fn dir(app:&AppHandle)->Result<PathBuf,String>{let p=app.path().app_data_dir().map_err(|e|e.to_string())?;fs::create_dir_all(&p).map_err(|e|e.to_string())?;Ok(p)}
fn path(app:&AppHandle,name:&str)->Result<PathBuf,String>{Ok(dir(app)?.join(name))}
pub fn load_settings(app:&AppHandle)->AppSettings{let Ok(p)=path(app,"settings.json")else{return AppSettings::default()};let Ok(raw)=fs::read_to_string(&p)else{return AppSettings::default()};match serde_json::from_str::<AppSettings>(&raw){Ok(x)=>x.normalized(),Err(e)=>{log::warn!("Invalid settings; using defaults: {e}");let _=fs::rename(&p,p.with_extension("json.corrupt"));AppSettings::default()}}}
pub fn save_settings(app:&AppHandle,x:&AppSettings)->Result<(),String>{let raw=serde_json::to_string_pretty(x).map_err(|e|e.to_string())?;fs::write(path(app,"settings.json")?,raw).map_err(|e|e.to_string())}
pub fn load_history(app:&AppHandle,enabled:bool,max:usize)->Vec<ClipboardItem>{if !enabled{return vec![]}let Ok(p)=path(app,"history.json")else{return vec![]};let Ok(raw)=fs::read_to_string(&p)else{return vec![]};match serde_json::from_str::<Vec<ClipboardItem>>(&raw){Ok(mut x)=>{x.truncate(max);x},Err(e)=>{log::warn!("Invalid history; ignoring: {e}");let _=fs::rename(&p,p.with_extension("json.corrupt"));vec![]}}}
pub fn save_history(app:&AppHandle,enabled:bool,items:&[ClipboardItem])->Result<(),String>{let p=path(app,"history.json")?;if !enabled{let _=fs::remove_file(p);return Ok(())}let raw=serde_json::to_string(items).map_err(|e|e.to_string())?;fs::write(p,raw).map_err(|e|e.to_string())}
pub fn clear_history_file(app:&AppHandle){if let Ok(p)=path(app,"history.json"){let _=fs::remove_file(p);}}
