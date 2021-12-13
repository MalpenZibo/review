use crate::{AnyComponent, Events, HookContext, Tag, VNode};
use std::collections::HashMap;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub(crate) struct Element {
    pub dom: Option<web_sys::Element>,
    pub tag: Tag,
    pub attributes: HashMap<String, String>,
    pub events: Events,
    pub unprocessed_children: Vec<VNode>,
}

impl Element {
    pub(crate) fn update_element_dom(
        &mut self,
        attributes: HashMap<String, String>,
        events: Events,
    ) {
        let attributes_to_set =
            attributes
                .iter()
                .filter_map(|(k, v)| match self.attributes.get(k) {
                    Some(old_v) if old_v != v => Some((k, v)),
                    None => Some((k, v)),
                    _ => None,
                });
        let attributes_to_remove = self.attributes.keys().filter_map(|k| {
            if !attributes.contains_key(k) {
                Some(k)
            } else {
                None
            }
        });
        for a in attributes_to_set {
            if let Some(dom) = &self.dom {
                dom.set_attribute(&a.0, &a.1).expect("set attribute error");
            }
        }
        for a in attributes_to_remove {
            if let Some(dom) = &self.dom {
                dom.remove_attribute(a).expect("set attribute error");
            }
        }
        self.attributes = attributes;

        let events_to_set = events
            .0
            .iter()
            .filter_map(|(k, v)| match self.events.0.get(k) {
                Some(old_v) => Some((Some(old_v), k, v)),
                None => Some((None, k, v)),
            });
        let events_to_remove = self.events.0.iter().filter_map(|(k, v)| {
            if !events.0.contains_key(k) {
                Some((k, v))
            } else {
                None
            }
        });
        for (old_event, event_type, new_event) in events_to_set {
            if let Some(dom) = &self.dom {
                if let Some(old_event) = old_event {
                    dom.remove_event_listener_with_callback(
                        event_type.as_ref(),
                        old_event.as_ref().as_ref().unchecked_ref(),
                    )
                    .expect("remove event error");
                }
                dom.add_event_listener_with_callback(
                    event_type.as_ref(),
                    new_event.as_ref().as_ref().unchecked_ref(),
                )
                .expect("add event error");
            }
        }
        for (event_type, old_event) in events_to_remove {
            if let Some(dom) = &self.dom {
                dom.remove_event_listener_with_callback(
                    event_type.as_ref(),
                    old_event.as_ref().as_ref().unchecked_ref(),
                )
                .expect("remove event error");
            }
        }
        self.events = events;
    }
}

#[derive(Debug)]
pub(crate) struct Text {
    pub dom: Option<web_sys::Text>,
    pub text: String,
}

impl Text {
    pub(crate) fn update_text_dom(&mut self, new_text: String) {
        if let Some(dom) = &self.dom {
            dom.set_data(&new_text);
        }
        self.text = new_text;
    }
}

#[derive(Debug)]
pub(crate) struct Component {
    pub hooks: HookContext,
    pub function: Box<dyn AnyComponent>,
}

#[derive(Debug)]
pub(crate) enum Node {
    Element(Element),
    Text(Text),
    Component(Component),
}

impl Node {
    pub(crate) fn create_dom(&mut self, document: Option<&web_sys::Document>) {
        if let Some(document) = document {
            match self {
                Node::Element(Element {
                    dom,
                    tag,
                    attributes,
                    events,
                    ..
                }) => {
                    dom.replace(document.create_element(tag.as_ref()).unwrap());
                    for a in attributes {
                        if let Some(dom) = dom {
                            dom.set_attribute(&a.0, &a.1).expect("set attribute error");
                        }
                    }
                    for (event_type, event) in &events.0 {
                        if let Some(dom) = dom {
                            dom.add_event_listener_with_callback(
                                event_type.as_ref(),
                                event.as_ref().as_ref().unchecked_ref(),
                            )
                            .expect("add event error");
                        }
                    }
                }
                Node::Text(Text { dom, text }) => {
                    dom.replace(document.create_text_node(&text));
                }
                _ => {}
            }
        }
    }
}
