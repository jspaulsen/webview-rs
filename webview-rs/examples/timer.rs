//#![windows_subsystem = "windows"]

extern crate webview;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use webview::*;

fn main() {
    let counter = Arc::new(Mutex::new(0));

    let counter_inner = counter.clone();
    let webview = webview::WebViewBuilder::new()
        .title("Timer example")
        .content(Content::Html(HTML))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(0)
        .invoke_handler(|webview, arg| {
            match arg {
                "reset" => {
                    let mut counter = counter.lock().unwrap();
                    let user_data = webview.user_data();
                    let mut lock = user_data.write().unwrap();
                    *lock += 10;

                    drop(lock);
                    *counter = 0;
                    render(webview, *counter)?;
                }
                "exit" => {
                    webview.terminate();
                }
                _ => unimplemented!(),
            };
            Ok(())
        })
        .build()
        .unwrap();

    let handle = webview.handle();
    thread::spawn(move || loop {
        {
            let mut counter = counter_inner.lock().unwrap();
            *counter += 1;
            let count = *counter;
            handle
                .dispatch(move |webview| {
                    let user_data = webview.user_data();
                    let mut lock = user_data.write().unwrap();
                    *lock -= 1;
                    drop(lock);
                    render(webview, count)
                })
                .unwrap();
        }
        thread::sleep(Duration::from_secs(1));
    });

    webview.run().unwrap();
}

fn render(webview: &mut WebView<i32>, counter: u32) -> WVResult {
    let data = *webview.user_data().read().unwrap();
    println!("counter: {}, userdata: {}", counter, data);
    webview.eval(&format!("updateTicks({}, {})", counter, data))
}

const HTML: &str = r#"
<!doctype html>
<html>
	<body>
		<p id="ticks"></p>
		<button onclick="external.invoke('reset')">reset</button>
		<button onclick="external.invoke('exit')">exit</button>
		<script type="text/javascript">
			function updateTicks(n, u) {
				document.getElementById('ticks').innerHTML = 'ticks ' + n + '<br>' + 'userdata ' + u;
			}
		</script>
	</body>
</html>
"#;
