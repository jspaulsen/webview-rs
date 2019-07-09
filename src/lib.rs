// //! [![Build Status]][travis] [![Latest Version]][crates.io]
// //!
// //! [Build Status]: https://api.travis-ci.org/Boscop/web-view.svg?branch=master
// //! [travis]: https://travis-ci.org/Boscop/web-view
// //! [Latest Version]: https://img.shields.io/crates/v/web-view.svg
// //! [crates.io]: https://crates.io/crates/web-view
// //!
// //! This library provides Rust bindings for the [webview](https://github.com/zserge/webview) library
// //! to allow easy creation of cross-platform Rust desktop apps with GUIs based on web technologies.
// //!
// //! It supports two-way bindings for communication between the Rust backend and JavaScript frontend.
// //!
// //! It uses Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) on Windows, so your app
// //! will be **much** leaner than with Electron.
// //!
// //! To use a custom version of webview, define an environment variable WEBVIEW_DIR with the path to
// //! its source directory.
// //!
// //! For usage info please check out [the examples] and the [original readme].
// //!
// //! [the examples]: https://github.com/Boscop/web-view/tree/master/examples
// //! [original readme]: https://github.com/zserge/webview/blob/master/README.md
//
extern crate urlencoding;
extern crate webview_sys as ffi;

mod color;
mod content;
mod dialog;
mod error;
//mod escape;
mod handle;
mod user_data;

use color::Color;
pub use content::Content;
use dialog::DialogBuilder;
pub use error::{
    Error,
    WVResult,
};
use ffi::*;
use handle::Handle;
use std::ffi::{
    CStr,
    CString,
};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::os::raw::*;
use std::ptr::drop_in_place;
use std::sync::{
    Arc,
    RwLock,
};
use user_data::UserData;
use urlencoding::encode;


#[derive(Clone)]
pub struct WebView<T> {
    internal: *mut WebViewFFI,
    _phantom: PhantomData<T>
}

/// [`WebView`]: struct.WebView.html
pub struct WebViewBuilder<'a, T: 'a, I, C: AsRef<str>> {
    pub title: &'a str,
    pub content: Option<Content<C>>,
    pub width: i32,
    pub height: i32,
    pub resizable: bool,
    pub debug: bool,
    pub invoke_handler: Option<I>, // req
    pub user_data: Option<T> //Option<Box<UserData<'a, T>>>, // req
}

impl<'a, T: 'a, I, C> Default for WebViewBuilder<'a, T, I, C>
where
    I: FnMut(&mut WebView<T>, &str) -> WVResult + 'a,
    C: AsRef<str>,
{
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let debug = true;
        #[cfg(not(debug_assertions))]
        let debug = false;

        WebViewBuilder {
            title: "Application",
            content: None,
            width: 800,
            height: 600,
            resizable: true,
            debug,
            invoke_handler: None,
            user_data: None,
        }
    }
}

impl<'a, T: 'a, I, C> WebViewBuilder<'a, T, I, C>
where
    I: FnMut(&mut WebView<T>, &str) -> WVResult + 'a,
    C: AsRef<str>,
{
    /// Alias for [`WebViewBuilder::default()`].
    ///
    /// [`WebViewBuilder::default()`]: struct.WebviewBuilder.html#impl-Default
    pub fn new() -> Self {
        WebViewBuilder::default()
    }

    /// Sets the title of the WebView window.
    ///
    /// Defaults to `"Application"`.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Sets the content of the WebView. Either a URL or a HTML string.
    pub fn content(mut self, content: Content<C>) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets the size of the WebView window.
    ///
    /// Defaults to 800 x 600.
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the resizability of the WebView window. If set to false, the window cannot be resized.
    ///
    /// Defaults to `true`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Enables or disables debug mode.
    ///
    /// Defaults to `true` for debug builds, `false` for release builds.
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Sets the invoke handler callback. This will be called when a message is received from
    /// JavaScript.
    ///
    /// # Errors
    ///
    /// If the closure returns an `Err`, it will be returned on the next call to [`step()`].
    ///
    /// [`step()`]: struct.WebView.html#method.step
    pub fn invoke_handler(mut self, invoke_handler: I) -> Self {
        self.invoke_handler = Some(invoke_handler);
        self
    }

    /// Sets the initial state of the user data. This is an arbitrary value stored on the WebView
    /// thread, accessible from dispatched closures without synchronization overhead.
    pub fn user_data(mut self, data: T) -> Self {
        self.user_data = Some(data); //Some(Box::new(UserData::new(data)));
        self
    }

    /// Validates provided arguments and returns a new WebView if successful.
    pub fn build(self) -> WVResult<WebView<T>> {
        macro_rules! require_field {
            ($name:ident) => {
                self.$name
                    .ok_or_else(|| Error::UninitializedField(stringify!($name)))?
            };
        }

        let title = CString::new(self.title)?;
        let content = require_field!(content);
        let user_data = require_field!(user_data);
        let invoke_handler = require_field!(invoke_handler);
        let url = match content {
            Content::Url(url) => CString::new(url.as_ref())?,
            Content::Html(html) => CString::new(format!("data:text/html,{}", encode(html.as_ref())))?
        };
        let data = Box::new(
            UserData::new(
                user_data,
                Box::new(invoke_handler),
            )
        );

        WebView::new(
            WebViewFFI::new(
                url.as_ptr(),
                title.as_ptr(),
                self.width,
                self.height,
                self.resizable,
                self.debug,
                ffi_invoke_handler::<T>,
                Box::into_raw(data) as _,
            )
        )
    }
}

impl<T> WebView<T> {
    pub fn new(internal: WebViewFFI) -> WVResult<Self> {
        unsafe {
            let raw = Box::into_raw(Box::new(internal));

            match ffi::webview_init(raw) {
                0 => {
                    Ok(
                        Self {
                            internal: raw,
                            _phantom: PhantomData,
                        }
                    )
                },
                _ => Err(Error::Initialization),
            }
        }
    }

    pub fn from_ptr(internal: *mut WebViewFFI) -> Self {
        Self {
            internal,
            _phantom: PhantomData,
        }
    }

    pub fn user_data(&self) -> Arc<RwLock<T>> {
        unsafe {
            let ffi: &WebViewFFI = &*self.internal;
            let data: &UserData<T> = &* (ffi.userdata as *mut UserData<T>);

            data.user_data()
        }
    }

    // TODO: will change
    fn state_data_mut(&mut self) -> &mut UserData<T> {
        unsafe {
            let ffi: &WebViewFFI = &*self.internal;
            &mut *(ffi.userdata as *mut UserData<T>)

        }
    }

    fn step(&mut self) -> Option<WVResult> {
        unsafe {
            match webview_loop(self.internal, 1) {
                0 => {
                    match self.state_data_mut().result.take() {
                        Some(r) => Some(r),
                        None => Some(Ok(())),
                    }
                },
                _ => None,
            }
        }
    }

    pub fn run(mut self) -> WVResult {
        loop {
            match self.step() {
                Some(e) => e?,
                None => return Ok(())
            }
        }
    }

    pub fn terminate(&mut self) {
        unsafe { webview_terminate(self.internal) }
    }

    pub fn eval(&mut self, js: &str) -> WVResult {
        let js = CString::new(js)?;
        let ret = unsafe {
            webview_eval(self.internal, js.as_ptr())
        };

        match ret {
            0 => Ok(()),
            _ => Err(Error::JsEvaluation),
        }
    }

    pub fn inject_css(&mut self, js: &str) -> WVResult {
        let js = CString::new(js)?;
        let ret = unsafe {
            webview_inject_css(self.internal, js.as_ptr())
        };

        match ret {
            0 => Ok(()),
            _ => Err(Error::CssInjection),
        }
    }

    pub fn set_color<C: Into<Color>>(&mut self, color: C) {
        let color = color.into();

        unsafe {
            webview_set_color(
                self.internal,
                color.r,
                color.g,
                color.b,
                color.a,
            );
        }
    }

    pub fn set_title(&mut self, title: &str) -> WVResult {
        let title = CString::new(title)?;

        unsafe {
            webview_set_title(
                self.internal,
                title.as_ptr(),
            )
        };

        Ok(())
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe {
            webview_set_fullscreen(
                self.internal,
                fullscreen as _,
            )
        }
    }

    /// Returns a builder for opening a new dialog window.
    pub fn dialog(&mut self) -> DialogBuilder {
        DialogBuilder::new(self.internal)
    }

    pub fn handle(&self) -> Handle<T> {
        unsafe {
            let ffi: &WebViewFFI = &*self.internal;
            let user_data: &UserData<T> = & *(ffi.userdata as *mut UserData<T>);

            Handle::new(
                self.internal,
                user_data.live_lock(),
            )
        }
    }
}

impl<T> Drop for WebView<T> {
    fn drop(&mut self) {
        println!("Issa WebView drop");
        unsafe {
            let ffi: &WebViewFFI = &*self.internal;

            webview_exit(self.internal);

            // Drop both UserData and WebViewFFI which were
            // instantiated and made into primitive pointers for FFI
            drop_in_place(ffi.userdata as *mut UserData<T>);
            drop_in_place(self.internal);
        }
    }
}

pub unsafe extern "C" fn ffi_dispatch_handler<F, T>(ffi: *mut WebViewFFI, arg: *mut c_void)
where
    F: FnOnce(&mut WebView<T>) -> WVResult + Send + 'static,
{
    let mut webview = ManuallyDrop::new(WebView::<T>::from_ptr(ffi));
    let webffi: &WebViewFFI = &*ffi;
    let data: &mut UserData<T> = &mut *(webffi.userdata as *mut UserData<T>);

    data.result = Some({
        let closure: Box<F> = Box::from_raw(arg as _);
        (*closure)(&mut webview)
    });
}

pub unsafe extern "C" fn ffi_invoke_handler<T>(ffi: *mut WebViewFFI, arg: *const c_char) {
    let webffi: &WebViewFFI = &*ffi;
    let arg = CStr::from_ptr(arg).to_string_lossy().to_string();
    let mut webview = ManuallyDrop::new(WebView::<T>::from_ptr(ffi));
    let data: &mut UserData<T> = &mut *(webffi.userdata as *mut UserData<T>);

    data.result = Some({
        (data.invoke_handler)(&mut webview, &arg)
    });
}

#[cfg(test)]
mod test {
    // use ffi::WebViewFFI;
    // use std::ffi::c_void;
    // use std::os::raw::*;
    // use super::*;


    // #[no_mangle]
    // pub extern "C" fn webview_init(webview: *mut WebViewFFI) -> c_int {
    //     127
    // }

    #[test]
    fn do_something() {
        // let view = WebViewBuilder::new()
        //     .title("Dummy")
        //     .content(Content::Url("https://dummy-url"))
        //     .user_data(())
        //     .invoke_handler(|_, _| Ok(()))
        //     .build();
        //
        // println!("Wow!: {:?}", view.is_err());
    }

    // extern {
    //     //pub fn webview_init(webview: *mut WebViewFFI) -> c_int;
    // 	pub fn webview_loop(webview: *mut WebViewFFI, blocking: c_int) -> c_int;
    // 	pub fn webview_terminate(webview: *mut WebViewFFI);
    // 	pub fn webview_exit(webview: *mut WebViewFFI);
    // 	pub fn webview_dispatch(webview: *mut WebViewFFI, f: Option<DispatchFn>, arg: *mut c_void);
    // 	pub fn webview_eval(webview: *mut WebViewFFI, js: *const c_char) -> c_int;
    // 	pub fn webview_inject_css(webview: *mut WebViewFFI, css: *const c_char) -> c_int;
    // 	pub fn webview_set_title(webview: *mut WebViewFFI, title: *const c_char);
    // 	pub fn webview_set_fullscreen(webview: *mut WebViewFFI, fullscreen: c_int);
    // 	pub fn webview_set_color(webview: *mut WebViewFFI, red: u8, green: u8, blue: u8, alpha: u8);
    // 	pub fn webview_dialog(webview: *mut WebViewFFI, dialog_type: DialogType, flags: DialogFlags, title: *const c_char, arg: *const c_char, result: *mut c_char, result_size: usize);
    // }
}
