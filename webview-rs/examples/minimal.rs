#![windows_subsystem = "windows"]

extern crate webview;

use webview::*;

fn main() {
    webview::WebViewBuilder::new()
        .title("Minimal webview example")
        .content(Content::Url("https://en.m.wikipedia.org/wiki/Main_Page"))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap()
        .run()
        .unwrap();
}
