use tauri::{AppHandle,Manager};
use crate::{permissions,selection,state::AppState};
#[cfg(any(target_os="windows",target_os="macos"))]
pub fn start(app:AppHandle){
    std::thread::spawn(move||{
        if cfg!(target_os="macos")&&!permissions::accessibility_granted(){log::info!("Global wheel capture unavailable until macOS Accessibility permission is granted; sticky mode remains available");return}
        let result=rdev::listen(move|event|{
            if let rdev::EventType::Wheel{delta_y,..}=event.event_type{
                if delta_y!=0{
                    let state=app.state::<AppState>();
                    if state.selector.lock().active{let _=selection::navigate(&app,if delta_y<0{1}else{-1});}
                }
            }
        });
        if let Err(e)=result{log::warn!("Global input listener stopped: {e:?}")}
    });
}
#[cfg(not(any(target_os="windows",target_os="macos")))] pub fn start(_app:AppHandle){}
