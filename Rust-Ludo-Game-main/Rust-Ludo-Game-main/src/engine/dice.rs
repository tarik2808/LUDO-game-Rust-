use rand::Rng;

pub fn roll() -> u8 {
    rand::thread_rng().gen_range(1..7)
}
