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

    let unsafe_metrics = calls.unsafe_metrics.borrow_mut().take().unwrap();
    let total_unsafe: usize = unsafe_metrics.spans.iter().map(|row| row.2).sum();
    println!("Unsafe lines: {}", total_unsafe);
}