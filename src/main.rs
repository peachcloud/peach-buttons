use std::process;

use log::error;

fn main() {
    env_logger::init();

    if let Err(e) = peach_buttons::run() {
        error!("Application error: {}", e);
        process::exit(1);
    }
}
