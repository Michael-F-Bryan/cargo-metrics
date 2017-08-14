#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;
extern crate syntax;
extern crate getopts;
extern crate rustc_errors as errors;

use rustc_driver::CompilerCalls;
use rustc_driver::driver::CompileController;
use rustc::session::Session;


pub struct Calls;

impl<'a> CompilerCalls<'a> for Calls {
    fn build_controller(&mut self, _: &Session, _: &getopts::Matches) -> CompileController<'a> {
        panic!("TODO: construct a CompileController")
    }
}


pub fn run(args: &[String]) {
    let mut calls = Calls;
    let (compile_result, _session) = rustc_driver::run_compiler(args, &mut calls, None, None);
    compile_result.unwrap();
}