use crate::game::SquareValue;
use crate::square::Square;
use crate::square::SquareProps;
use bom::children;
use bom::component;
use bom::ElementBuilder;
use bom::Tag::Div;
use std::rc::Rc;

pub struct BoardProps {
    pub squares: [Option<SquareValue>; 9],
    pub on_click: Rc<dyn Fn(usize)>,
}

#[component(Board)]
pub fn board(props: &BoardProps) -> VNode {
    let square = |index: usize| {
        let on_click = props.on_click.clone();

        Square(SquareProps {
            value: match props.squares.get(index) {
                Some(v) => *v,
                _ => None,
            },
            on_click: Rc::new(move || on_click(index)),
        })
    };

    let row = |start: usize| {
        Div.with_attribute("class", "board-row")
            .with_children(children!(
                square(start),
                square(start + 1),
                square(start + 2)
            ))
    };

    Div.with_children(children!(row(0), row(3), row(6))).into()
}
