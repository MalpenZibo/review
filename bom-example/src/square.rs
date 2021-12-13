use crate::game::SquareValue;
use bom::callback;
use bom::component;
use bom::ElementBuilder;
use bom::EventType::OnClick;
use bom::Tag::Button;
use std::rc::Rc;

pub struct SquareProps {
    pub value: Option<SquareValue>,
    pub on_click: Rc<dyn Fn() -> ()>,
}

#[component(Square)]
pub fn square(props: &SquareProps) -> VNode {
    let c = props.on_click.clone();

    Button
        .with_attribute("class", "square")
        .with_event(OnClick, callback!(move || c()))
        .with_child(if let Some(value) = props.value {
            format!("{}", value)
        } else {
            "".into()
        })
        .into()
}
