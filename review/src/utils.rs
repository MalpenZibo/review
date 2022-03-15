use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub fn init_logger() {
    if cfg!(feature = "console_log") {
        use log::Level;
        console_log::init_with_level(Level::Trace).expect("error initializing log");
        if cfg!(feature = "log-panics") {
            log_panics::init();
        }
    }
}

pub(crate) fn request_idle_callback(f: &Closure<dyn FnMut(web_sys::IdleDeadline)>) {
    web_sys::window()
        .expect("window access error")
        .request_idle_callback(f.as_ref().unchecked_ref())
        .expect("should register `requestIdleCallback` OK");
}
