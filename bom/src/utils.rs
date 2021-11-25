use crate::tag::Tag;
use crate::Events;
use std::collections::HashMap;
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

pub(crate) fn create_element_dom(
    tag: &Tag,
    attributes: &HashMap<String, String>,
    events: &Events,
    document: Option<&web_sys::Document>,
) -> Option<web_sys::Element> {
    if let Some(document) = document {
        let element = document.create_element(tag.as_ref()).unwrap();
        for (key, value) in attributes.iter() {
            element.set_attribute(key, value).unwrap();
        }
        for (key, value) in events.0.iter() {
            element
                .add_event_listener_with_callback(
                    key.as_ref(),
                    value.as_ref().as_ref().unchecked_ref(),
                )
                .unwrap();
        }
        Some(element)
    } else {
        None
    }
}

pub(crate) fn create_text_dom(
    text: &str,
    document: Option<&web_sys::Document>,
) -> Option<web_sys::Text> {
    if let Some(document) = document {
        Some(document.create_text_node(&text))
    } else {
        None
    }
}
