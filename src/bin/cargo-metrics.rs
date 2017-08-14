extern crate cargo_metrics;

use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    cargo_metrics::run(&args);
}