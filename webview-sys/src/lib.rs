// TODO: Unsure if wildcard BSD _actually_works
#[cfg(any(target_os = "linux", target_os = "*bsd"))]
mod webview_bind;
#[cfg(target_os = "macos")]
mod webview_bind_osx;
#[cfg(target_os = "windows")]
mod webview_bind_win;

mod webview_ffi;

#[macro_use]
extern crate bitflags;

pub use webview_ffi::*;


#[repr(C)]
pub enum DialogType {
	Open  = 0,
	Save  = 1,
	Alert = 2,
}

bitflags! {
	#[repr(C)]
	pub struct DialogFlags: u32 {
		const FILE      = 0b0000;
		const DIRECTORY = 0b0001;
		const INFO      = 0b0010;
		const WARNING   = 0b0100;
		const ERROR     = 0b0110;
	}
}
