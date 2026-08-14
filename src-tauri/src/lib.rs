mod clipboard; mod commands; mod global_input; mod history; mod models; mod overlays; mod permissions; mod selection; mod settings_store; mod shortcuts; mod state; mod tray;
use history::ClipboardHistory;
use state::AppState;
use tauri::{Manager,WindowEvent};

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build())
        .plugin(shortcuts::plugin())
        .setup(|app|{
            app.handle().plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent,Some(vec!["--hidden"])))?;
            let settings=settings_store::load_settings(app.handle());
            let history=ClipboardHistory::from_items(settings_store::load_history(app.handle(),settings.history.persist_history,settings.history.max_items),settings.history.max_items);
            app.manage(AppState::new(settings.clone(),history));
            shortcuts::register_all(app.handle()).map_err(std::io::Error::other)?;
            tray::build(app.handle()).map_err(std::io::Error::other)?;
            clipboard::start(app.handle().clone()); global_input::start(app.handle().clone());
            if settings.first_run_completed&&settings.general.start_hidden{if let Some(w)=app.get_webview_window("settings"){let _=w.hide();}}
            Ok(())
        })
        .on_window_event(|window,event|{if window.label()=="settings"{if let WindowEvent::CloseRequested{api,..}=event{api.prevent_close();let _=window.hide();}}})
        .invoke_handler(tauri::generate_handler![commands::get_settings,commands::get_history,commands::save_settings,commands::clear_history,commands::select_history_item,commands::navigate_selection,commands::accept_selection,commands::cancel_selection,commands::show_history,commands::open_settings,commands::toggle_monitoring,commands::platform_status,commands::complete_first_run,commands::reset_settings,commands::quit_app])
        .build(tauri::generate_context!()).expect("failed to build Clipboard Preview")
        .run(|handle,event|{if let tauri::RunEvent::ExitRequested{..}=event{let state=handle.state::<AppState>();let s=state.settings.read().clone();if s.history.clear_on_exit{settings_store::clear_history_file(handle);}else if s.history.persist_history{let items=state.history.lock().items();let _=settings_store::save_history(handle,true,&items);}}});
}
