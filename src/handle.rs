use ::{
    ffi_dispatch_handler,
    WebView,
};
use error::{
    Error,
    WVResult,
};
use ffi::{
    webview_dispatch,
    WebViewFFI,
};
use std::marker::PhantomData;
use std::sync::{
    Arc,
    RwLock,
    Weak,
};

pub struct Handle<T> {
    internal: *mut WebViewFFI,
    live: Weak<RwLock<()>>,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(internal: *mut WebViewFFI, live: Arc<RwLock<()>>) -> Self {
        Self {
            internal,
            live: Arc::downgrade(&live),
            _phantom: PhantomData,
        }
    }

    pub fn dispatch<F>(&self, func: F) -> WVResult
    where
        F: FnOnce(&mut WebView<T>) -> WVResult + Send + 'static,
    {
        let lock = self.live
            .upgrade()
            .ok_or(Error::Dispatch)?;
        let _locked = lock.read().map_err(|_| Error::Dispatch)?;
        let closure = Box::new(func);

        unsafe {
            webview_dispatch(
                self.internal,
                Some(ffi_dispatch_handler::<F, T>),
                Box::into_raw(closure) as _,
            )
        }

        println!("dispatch!");
        Ok(())
    }
}

unsafe impl<T> Send for Handle<T> {}
