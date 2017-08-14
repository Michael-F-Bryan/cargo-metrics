use getopts;
use syntax::visit;
use rustc::session::Session;
use rustc_driver::driver::{CompileController, CompileState};
use rustc_driver::{CompilerCalls, Compilation};
use std::rc::Rc;
use std::cell::RefCell;

use unsafe_visitor::{Location, Metrics, UnsafeVisitor};
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
            let mut visitor = UnsafeVisitor::new();

            // analyse the crate
            visit::walk_crate(&mut visitor, ast);

            // then resolve spans to line numbers and locations
            let codemap = compile_state.session.codemap();
            let spans = visitor
                .unsafe_lines
                .iter()
                .map(|span| {
                    let start = Location::from(codemap.lookup_char_pos(span.lo));
                    let end = Location::from(codemap.lookup_char_pos(span.hi));
                    let diff = end.line - start.line;

                    (start, end, if diff == 0 { 1 } else { diff })
                })
                .collect();

            *metrics.borrow_mut() = Some(Metrics { spans: spans });
        };

        controller
    }
}
