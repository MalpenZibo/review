use crate::board::Board;
use crate::board::BoardProps;
use bom::callback;
use bom::children;
use bom::component;
use bom::use_state;
use bom::ElementBuilder;
use bom::EventType::OnClick;
use bom::Tag::Button;
use bom::Tag::Div;
use bom::Tag::Li;
use bom::Tag::Ol;
use bom::VNode;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SquareValue {
    X,
    O,
}
impl Display for SquareValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SquareValue::X => write!(f, "X"),
            SquareValue::O => write!(f, "O"),
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    squares_history: Vec<[Option<SquareValue>; 9]>,
    step_index: usize,
    x_is_next: bool,
}

fn calculate_winner(squares: &[Option<SquareValue>]) -> Option<SquareValue> {
    let lines = [
        [0, 1, 2],
        [3, 4, 5],
        [6, 7, 8],
        [0, 3, 6],
        [1, 4, 7],
        [2, 5, 8],
        [0, 4, 8],
        [2, 4, 6],
    ];

    for i in 0..lines.len() {
        let [a, b, c] = lines[i];
        match (squares.get(a), squares.get(b), squares.get(c)) {
            (Some(Some(a)), Some(Some(b)), Some(Some(c))) if a == b && a == c => return Some(*a),
            _ => {}
        }
    }

    None
}

#[component(Game)]
pub fn game() -> VNode {
    let game_state = use_state(GameState {
        squares_history: vec![[None; 9]],
        step_index: 0,
        x_is_next: true,
    });

    let current = game_state.value.squares_history[game_state.value.step_index];

    let moves: Vec<VNode> = game_state
        .value
        .squares_history
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let desc = if i > 0 {
                format!("Go to move # {}", i)
            } else {
                format!("Go to game start")
            };
            Li.with_child(Button.with_child(desc).with_event(OnClick, {
                let game_state = game_state.clone();
                callback!(move || game_state.set(GameState {
                    squares_history: game_state.value.squares_history.clone(),
                    step_index: i,
                    x_is_next: i % 2 == 0
                }))
            }))
            .into()
        })
        .collect();

    let status: VNode = match calculate_winner(&current) {
        Some(winner) => format!("Winner {}", winner).into(),
        _ => if current.iter().any(|s| s.is_none()) {
            format!(
                "Next player: {}",
                if game_state.value.x_is_next {
                    SquareValue::X
                } else {
                    SquareValue::O
                }
            )
        } else {
            "Tie".to_owned()
        }
        .into(),
    };

    let handle_click = {
        let game_state = game_state.clone();
        move |index: usize| {
            let mut new_square = current.clone();
            if calculate_winner(&new_square).is_none() {
                if let Some(square @ None) = new_square.get_mut(index) {
                    *square = Some(if game_state.value.x_is_next {
                        SquareValue::X
                    } else {
                        SquareValue::O
                    });

                    let mut new_history = game_state.value.squares_history
                        [0..game_state.value.step_index + 1]
                        .to_vec();
                    new_history.push(new_square);
                    game_state.set(GameState {
                        squares_history: new_history,
                        step_index: game_state.value.step_index + 1,
                        x_is_next: !game_state.value.x_is_next,
                    });
                }
            }
        }
    };

    Div.with_attribute("class", "game")
        .with_children(children!(
            Div.with_attribute("game", "game-board").with_child(Board({
                BoardProps {
                    squares: current,
                    on_click: Rc::new(handle_click),
                }
            })),
            Div.with_attribute("class", "game-info")
                .with_children(children!(Div.with_child(status), Ol.with_children(moves)))
        ))
        .into()
}
