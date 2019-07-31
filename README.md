# WebView
This library provides Rust bindings for the [webview](https://github.com/zserge/webview) library to allow easy creation of cross-platform Rust desktop apps with GUIs based on web technologies.

It supports two-way bindings for communication between the Rust backend and JavaScript frontend.

It uses Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) on Windows, so your app will be **much** leaner than with Electron.

For usage info please check out [the examples](../../tree/master/examples) and the [original readme](https://github.com/zserge/webview/blob/master/README.md).

### Note
This library is derived from [web-view](https://github.com/Boscop/web-view); original library has data safety issues which would result in SIGABRT (as described [here](https://github.com/Boscop/web-view/issues/26), irrespective of thread status).
This fork attempts to solve those issues with a rewrite to (_hopefully_) better, more idiomatic patterns and introduces tests (where possible).

There are (breaking) API changes between the two versions, mainly `user_data` (which is now guarded by a `RwLock`).

## Tests
Unit tests are not thread safe!  Tests should be run as `cargo test -- --test-threads=1` or via `make tests`
