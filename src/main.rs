mod app;
#[allow(clippy::wildcard_imports)]
use app::*;
#[allow(clippy::wildcard_imports)]
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <App/>
        }
    });
}
