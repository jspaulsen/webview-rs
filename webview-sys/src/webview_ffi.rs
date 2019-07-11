use std::ffi::c_void;
use std::mem;
use std::os::raw::*;
use crate::{
    DialogFlags,
    DialogType,
};

#[cfg(any(target_os = "linux", target_os = "*bsd"))]
use crate::webview_bind::webview_priv;
#[cfg(target_os = "macos")]
use crate::webview_bind_osx::webview_priv;
#[cfg(target_os = "windows")]
use crate::webview_bind_win::webview_priv;


pub type InvokeFn = unsafe extern "C" fn(WebViewFFI: *mut WebViewFFI, arg: *const c_char);
pub type DispatchFn = unsafe extern "C" fn(WebViewFFI: *mut WebViewFFI, arg: *mut c_void);

#[repr(C)]
pub struct WebViewFFI {
    pub url: *const c_char,
    pub title: *const c_char,
    pub width: c_int,
    pub height: c_int,
    pub resizable: c_int,
    pub debug: c_int,
    external_invoke_cb: Option<InvokeFn>,
    private: [u8; mem::size_of::<webview_priv>()],
    pub userdata: *mut c_void,
}

impl WebViewFFI {
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    pub fn new(url: *const c_char, title: *const c_char, width: i32, height: i32, resizable: bool, debug: bool, invoke_fn: InvokeFn, userdata: *mut c_void) -> WebViewFFI {
        Self {
            url,
            title,
            width,
            height,
            resizable: resizable as i32,
            debug: debug as i32,
            external_invoke_cb: Some(invoke_fn),
            private: [0; mem::size_of::<webview_priv>()],
            userdata,
        }
    }
}

impl Drop for WebViewFFI {
    fn drop(&mut self) {
        println!("Issa WebViewFFI drop");
    }
}

extern {
    pub fn webview_init(webview: *mut WebViewFFI) -> c_int;
	pub fn webview_loop(webview: *mut WebViewFFI, blocking: c_int) -> c_int;
	pub fn webview_terminate(webview: *mut WebViewFFI);
	pub fn webview_exit(webview: *mut WebViewFFI);
	pub fn webview_dispatch(webview: *mut WebViewFFI, f: Option<DispatchFn>, arg: *mut c_void);
	pub fn webview_eval(webview: *mut WebViewFFI, js: *const c_char) -> c_int;
	pub fn webview_inject_css(webview: *mut WebViewFFI, css: *const c_char) -> c_int;
	pub fn webview_set_title(webview: *mut WebViewFFI, title: *const c_char);
	pub fn webview_set_fullscreen(webview: *mut WebViewFFI, fullscreen: c_int);
	pub fn webview_set_color(webview: *mut WebViewFFI, red: u8, green: u8, blue: u8, alpha: u8);
	pub fn webview_dialog(webview: *mut WebViewFFI, dialog_type: DialogType, flags: DialogFlags, title: *const c_char, arg: *const c_char, result: *mut c_char, result_size: usize);
}
