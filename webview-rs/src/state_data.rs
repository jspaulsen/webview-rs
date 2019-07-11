use std::sync::{
    Arc,
    RwLock,
};
use crate::error::WVResult;
use crate::WebView;

pub struct WebViewStateData<'a, T> {
    live: Arc<RwLock<()>>,
    internal: Arc<RwLock<T>>,
    pub invoke_handler: Box<FnMut(&mut WebView<T>, &str) -> WVResult + 'a>,
    pub result: Option<WVResult>,
}

impl<'a, T> WebViewStateData<'a, T> {
    pub fn new(data: T, invoke_handler: Box<FnMut(&mut WebView<T>, &str) -> WVResult + 'a>) -> Self {
        Self {
            live: Arc::new(RwLock::new(())),
            internal: Arc::new(RwLock::new(data)),
            invoke_handler,
            result: None,
        }
    }

    pub fn user_data(&self) -> Arc<RwLock<T>> {
        self.internal.clone()
    }

    pub fn live_lock(&self) -> Arc<RwLock<()>> {
        self.live.clone()
    }
}
