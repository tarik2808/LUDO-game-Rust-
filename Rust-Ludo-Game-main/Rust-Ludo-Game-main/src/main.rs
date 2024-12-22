mod display;
mod engine;
mod game;

use game::LudoGame as Ludo;

fn main() {
    let mut ludo = Ludo::new();
    ludo.play();

    std::thread::sleep(std::time::Duration::from_secs(5));
}
