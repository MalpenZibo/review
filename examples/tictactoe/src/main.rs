mod board;
mod game;
mod square;

use game::Game;

fn main() {
    review::init_logger(review::log::Level::Debug);

    review::render(Game(()).into(), "root");
}
