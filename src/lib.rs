#![feature(rustc_private, box_syntax)]

extern crate rustc;
extern crate rustc_driver;
extern crate syntax;
extern crate syntax_pos;
extern crate getopts;
extern crate rustc_errors as errors;

mod calls;
mod unsafe_visitor;

use calls::Calls;


pub fn run(args: &[String]) {
    let mut calls = Calls::default();
    let (compile_result, _session) = rustc_driver::run_compiler(args, &mut calls, None, None);
    compile_result.unwrap();

    let metrics = calls.unsafe_metrics.borrow_mut().take().unwrap();
    let total_unsafe: usize = metrics.spans.iter().map(|row| row.num_lines).sum();
    let percentage = 100.0 * total_unsafe as f32 / metrics.total_lines as f32;
    println!("Unsafe lines: {}/{} ({:.2}%)", total_unsafe, metrics.total_lines, percentage);
}