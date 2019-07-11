use std::thread;
use webview::*;

const TERMINATE_HTML: &str = include_str!("fixtures/invoke_terminate.html");
const INJECT_CSS_HTML: &str = include_str!("fixtures/inject_css.html");


#[test]
fn test_dispatch_terminate() {
    let view = WebViewBuilder::new()
        .size(1, 1)
        .content(Content::Url("http://dummy.url"))
        .user_data(())
        .invoke_handler(|_, _| Ok(()))
        .build()
        .unwrap();
    let handle = view.handle();

    // Create thread with handle which creates a dispatch
    // which terminates the thread
    let thread = thread::spawn(move || {
        handle.dispatch(|view| {
            view.terminate();
            Ok(())
        }).unwrap();
    });

    view.run().unwrap();
    thread.join().unwrap();
}

#[test]
fn test_dispatch_invoke_terminate() {
    let view = WebViewBuilder::new()
        .size(1, 1)
        .content(Content::Html(TERMINATE_HTML))
        .user_data(())
        .invoke_handler(|view, arg| {
            match arg {
                "terminate" => {
                    view.terminate();
                },
                _ => panic!("Received unexpected arg {}", arg)
            }

            Ok(())
        })
        .build()
        .unwrap();
    let handle = view.handle();

    // Spawn thread which dispatch; dispatch calls eval
    // that triggers the javascript function in `invoke_terminate.html`
    // which triggers the invoke_handler and terminates the app
    let thread = thread::spawn(move || {
        handle.dispatch(|view| {
            view.eval("test_terminate()")
        }).unwrap();
    });

    view.run().unwrap();
    thread.join().unwrap();
}

#[test]
fn test_inject_css() {
    let view = WebViewBuilder::new()
        .size(1, 1)
        .content(Content::Html(INJECT_CSS_HTML))
        .user_data(())
        .invoke_handler(|view, arg| {
            println!("Invoke!");
            assert_eq!("4px", arg);

            view.terminate();
            Ok(())
        })
        .build()
        .unwrap();
    let handle = view.handle();

    // Spawn thread which dispatch; dispatch calls
    // inject_css and calls in the internal function which
    // returns the CSS element, asserts and terminates
    let thread = thread::spawn(move || {
        handle.dispatch(|view| {
            view.inject_css("#app { margin-left: 4px; }")?;
            view.eval("injected_css()")
        }).unwrap();
    });

    view.run().unwrap();
    thread.join().unwrap();
}


#[test]
fn test_window_size() {
    let width = 250;
    let height = 500;

    let view = WebViewBuilder::new()
        .size(width, height)
        .content(Content::Html(INJECT_CSS_HTML))
        .user_data(())
        .invoke_handler(|view, arg| {
            match arg {
                w if w.contains("width") => {
                    let actual_width: Vec<&str> = w.split(' ').collect();
                    assert_eq!(width, actual_width[1].parse::<i32>().unwrap());
                },
                h if h.contains("height") => {
                    let actual_width: Vec<&str> = h.split(' ').collect();
                    assert_eq!(height, actual_width[1].parse::<i32>().unwrap());
                },
                _ => panic!("Unexpected argument {}", arg)
            }

            view.terminate();
            Ok(())
        })
        .build()
        .unwrap();
    let handle = view.handle();

    // Spawn thread which dispatch; dispatch calls
    // invoke which provides the width & height from JS
    // which we compare to our expected values
    let thread = thread::spawn(move || {
        handle.dispatch(|view| {
            view.eval("external.invoke(`width ${window.innerWidth}`)")?;
            view.eval("external.invoke(`height ${window.innerHeight}`)")?;
            view.terminate();

            Ok(())
        }).unwrap();
    });

    view.run().unwrap();
    thread.join().unwrap();
}

#[test]
fn test_misc() {
    let view = WebViewBuilder::new()
        .size(1, 1)
        .content(Content::Url("http://dummy.url"))
        .user_data(())
        .invoke_handler(|_, _| Ok(()))
        .build()
        .unwrap();
    let handle = view.handle();

    // testing these is difficult, so just call and assert nothing explodes
    let thread = thread::spawn(move || {
        handle.dispatch(|view| {
            assert!(view.set_title("Some title").is_ok());
            view.set_color((0, 0, 0, 100));
            view.terminate();

            Ok(())
        }).unwrap();
    });

    view.run().unwrap();
    thread.join().unwrap();
}

// pub fn set_color<C: Into<Color>>(&mut self, color: C) {
//     let color = color.into();
//
//     unsafe {
//         webview_set_color(
//             self.internal,
//             color.r,
//             color.g,
//             color.b,
//             color.a,
//         );
//     }
// }
//
// pub fn set_title(&mut self, title: &str) -> WVResult {
//     let title = CString::new(title)?;
//
//     unsafe {
//         webview_set_title(
//             self.internal,
//             title.as_ptr(),
//         )
//     };
//
//     Ok(())
// }
