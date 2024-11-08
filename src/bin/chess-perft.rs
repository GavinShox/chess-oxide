use env_logger::{Builder, Env, Target};

use chess::perft;

fn main() {
    // initialise logger
    let mut builder = Builder::from_env(Env::default().default_filter_or("off"));
    builder.target(Target::Stdout);
    builder.init();
    perft(10, 5);
}
