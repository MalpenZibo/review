use crate::board::{Board, BoardProps};
use review::EventType::OnClick;
use review::Tag::{Button, Div, Li, Ul};
use review::{callback, children, component, use_state, ElementBuilder, VNode};
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

    for [a, b, c] in lines {
        match (squares.get(a), squares.get(b), squares.get(c)) {
            (Some(Some(a)), Some(Some(b)), Some(Some(c))) if a == b && a == c => return Some(*a),
            _ => {}
        }
    }

    None
}

#[component(Game)]
pub fn game() -> VNode {
    let (game_state, set_game_state) = use_state(GameState {
        squares_history: vec![[None; 9]],
        step_index: 0,
        x_is_next: true,
    });

    let current = game_state.squares_history[game_state.step_index];

    let moves: Vec<VNode> = game_state
        .squares_history
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let desc = if i > 0 {
                format!("Go to move # {}", i)
            } else {
                "Go to game start".to_string()
            };
            Li.with_child(Button.with_child(desc).with_event(OnClick, {
                let game_state = game_state.clone();
                let set_game_state = set_game_state.clone();
                callback!(move || set_game_state(GameState {
                    squares_history: game_state.squares_history.clone(),
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
                if game_state.x_is_next {
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
        let game_state = game_state;
        move |index: usize| {
            let mut new_square = current;
            if calculate_winner(&new_square).is_none() {
                if let Some(square @ None) = new_square.get_mut(index) {
                    *square = Some(if game_state.x_is_next {
                        SquareValue::X
                    } else {
                        SquareValue::O
                    });

                    let mut new_history =
                        game_state.squares_history[0..game_state.step_index + 1].to_vec();
                    new_history.push(new_square);
                    set_game_state(GameState {
                        squares_history: new_history,
                        step_index: game_state.step_index + 1,
                        x_is_next: !game_state.x_is_next,
                    });
                }
            }
        }
    };

    Div.with_attribute("class", "game")
        .with_children(children!(
            Div.with_attribute("game", "game-board")
                .with_child(Board(BoardProps {
                    squares: current,
                    on_click: Rc::new(handle_click),
                })),
            Div.with_attribute("class", "game-info")
                .with_children(children!(Div.with_child(status), Ul.with_children(moves)))
        ))
        .into()
}
