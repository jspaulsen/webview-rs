use crate::{
    ffi_dispatch_handler,
    WebView,
    WebViewFFI,
    webview_dispatch,
};
use crate::error::{
    Error,
    WVResult,
};
use std::marker::PhantomData;
use std::sync::{
    Arc,
    RwLock,
    Weak,
};


/// A thread-safe handle to a [`WebView`] instance. Used to dispatch closures onto its task queue.
///
/// [`WebView`]: struct.WebView.html
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

    /// Schedules a closure to be run on the [`WebView`] thread.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Dispatch`] if the [`WebView`] has been dropped.
    ///
    /// If the closure returns an `Err`, it will be returned on the next call to [`step()`].
    ///
    /// [`WebView`]: struct.WebView.html
    /// [`Error::Dispatch`]: enum.Error.html#variant.Dispatch
    /// [`step()`]: struct.WebView.html#method.step
    pub fn dispatch<F>(&self, func: F) -> WVResult
    where
        F: FnOnce(&mut WebView<T>) -> WVResult + Send + 'static,
    {
        let lock = self.live
            .upgrade()
            .ok_or(Error::Dispatch)?;
        let _locked = lock
            .read()
            .map_err(|_| Error::Dispatch)?;
        let closure = Box::new(func);

        unsafe {
            webview_dispatch(
                self.internal,
                Some(ffi_dispatch_handler::<F, T>),
                Box::into_raw(closure) as _,
            )
        }

        Ok(())
    }
}

unsafe impl<T> Send for Handle<T> {}
