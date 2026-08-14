#[cfg(target_os="macos")]
#[link(name="ApplicationServices",kind="framework")]
extern "C"{fn AXIsProcessTrusted()->std::ffi::c_uchar;}
#[cfg(target_os="macos")] pub fn accessibility_granted()->bool{unsafe{AXIsProcessTrusted()!=0}}
#[cfg(not(target_os="macos"))] pub fn accessibility_granted()->bool{true}
