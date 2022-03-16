mod board;
mod game;
mod square;

use game::Game;

fn main() {
    review::init_logger();

    review::render(Game(()).into(), "root");
}
