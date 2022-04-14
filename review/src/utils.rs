use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// Initialize the logger with the specified minimum log [log::Level]
pub fn init_logger(level: log::Level) {
    console_log::init_with_level(level).expect("error initializing log");
    if cfg!(feature = "log-panics") {
        log_panics::init();
    }
}

pub(crate) fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .expect("window access error")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
