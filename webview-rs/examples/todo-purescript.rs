#![windows_subsystem = "windows"]

extern crate webview;

use webview::*;

fn main() {
    webview::WebViewBuilder::new()
        .title("Rust / PureScript - Todo App")
        .content(Content::Html(include_str!("todo-ps/dist/bundle.html")))
        .size(320, 480)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap()
        .run()
        .unwrap();
}
