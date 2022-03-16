use crate::game::SquareValue;
use review::callback;
use review::component;
use review::ElementBuilder;
use review::EventType::OnClick;
use review::Tag::Button;
use std::rc::Rc;

pub struct SquareProps {
    pub value: Option<SquareValue>,
    pub on_click: Rc<dyn Fn()>,
}

#[component(Square)]
pub fn square(props: &SquareProps) -> VNode {
    let on_click = props.on_click.clone();
    Button
        .with_attribute("class", "square")
        .with_event(OnClick, callback!(move || on_click()))
        .with_child(if let Some(value) = props.value {
            format!("{}", value)
        } else {
            "".into()
        })
        .into()
}
