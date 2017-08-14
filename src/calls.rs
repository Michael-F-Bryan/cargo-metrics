use getopts;
use rustc::session::Session;
use rustc_driver::driver::{CompileController, CompileState};
use rustc_driver::{CompilerCalls, Compilation};
use std::rc::Rc;
use std::cell::RefCell;

use unsafe_visitor::{self, Metrics};
use std::marker::PhantomData;


#[derive(Default, Debug, Clone)]
pub struct Calls<'a> {
    /// Phantom data so we can ensure `Calls` is `'a`.
    _phantom: PhantomData<&'a ()>,
    pub unsafe_metrics: Rc<RefCell<Option<Metrics>>>,
}

impl<'a> CompilerCalls<'a> for Calls<'a> {
    fn build_controller(&mut self, _: &Session, _: &getopts::Matches) -> CompileController<'a> {
        let mut controller = CompileController::basic();
        controller.after_parse.stop = Compilation::Stop;

        let metrics = self.unsafe_metrics.clone();

        controller.after_parse.callback = box move |compile_state: &mut CompileState| {
            let ast = compile_state.krate.as_ref().unwrap();

            *metrics.borrow_mut() = Some(unsafe_visitor::analyse_ast(
                ast,
                compile_state.session.codemap(),
            ));
        };

        controller
    }
}
