# Counting Lines of Unsafe

The first metric we're going to calculate is the number of lines of unsafe
in a crate. For this we don't need to run any of the later steps like type
checking or `borrowck`, so we'll just use the AST generated after the compiler
finishes parsing. We can use the [`CompileController.after_parse`][cap] to 
hook in after the compiler has finished parsing and expanding macros.

For the actual AST inspection, the `syntax` crate provides an extremely handy
[Visitor] trait which will recursively visit each node in a [Crate]. So
that means creating an unsafe code counter will require:

- Creating a type which implements the [Visitor] trait, overriding just the
  methods which inspect `Block`s and `Item`s
- In the `Visitor` impl store the [Span] for all `unsafe` blocks we find
- Set the `controller.after_parse.callback` to be a closure which invokes our
  custom `Visitor`, making sure the result gets saved back in our `Calls` 
  struct
- In the top level `main()` function, print out the results of our `unsafe` 
  analysis

First, we'll create the custom `Visitor`.

```rust
# #![feature(rustc_private)]
# extern crate syntax;
use syntax::ast::{Block, BlockCheckMode, Item, ItemKind, Unsafety};
use syntax::visit::{self, Visitor};
use syntax::ext::quote::rt::Span;

pub struct UnsafeVisitor {
    unsafe_lines: Vec<Span>,
}

impl<'a> Visitor<'a> for UnsafeVisitor {
    fn visit_item(&mut self, item: &'a Item) {
        match item.node {
            ItemKind::Fn(_, Unsafety::Unsafe, ..) |
            ItemKind::Trait(Unsafety::Unsafe, ..) | 
            ItemKind::DefaultImpl(Unsafety::Unsafe, ..) | 
            ItemKind::Impl(Unsafety::Unsafe, ..) => {
                self.unsafe_lines.push(item.span.clone());
            }
            _ => {},
        }

        visit::walk_item(self, item);
    }

    fn visit_block(&mut self, block: &'a Block) {
        if let BlockCheckMode::Unsafe(_) = block.rules {
            self.unsafe_lines.push(block.span.clone());
        }

        visit::walk_block(self, block);
    }
}
# fn main() {}
```

It took a while to skim through all the docs for [Item] and [Block] to 
figure out how you can tell when something is `unsafe`, but that's more tedious
than difficult. For now we're going to assume you'll never get an unsafe block
inside a function which is also defined as unsafe.

Next, let's define a couple data structures to store our metrics in. They're
pretty boring as-is, just a couple *Plain Ol' Data* structs.

```rust
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub spans: Vec<Row>,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Row {
    pub start: Location,
    pub end: Location,
    pub num_lines: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Location {
    pub filename: String,
    pub line: usize,
    pub col: usize,
}
# fn main() {}
```

To make things easier, I've pulled the calling of `UnsafeVisitor` and resolving
all `Spans` into line numbers and file locations into a helper function called
`analyse_ast` which takes the AST and a `CodeMap` and transforms it into a 
`Metrics`.

The easiest way to return information from the internals of `rustc` to our top 
level is by updating the internal state of our `Calls` to hold the data to be
returned. Because the trait definition for `CompilerCalls` doesn't ensure our
`Calls` will outlive the running of `rustc_driver`, we need to use a 
`Rc<RefCell<T>>` to satisfy the borrow checker at runtime instead. To do things
properly, `Calls` will contain a `Rc<RefCell<Option<Metrics>>>`. The type 
definition looks quite intimidating, but basically it signifies that we'll only
have metrics after the compiler has run, and we wrap it in a `Rc<RefCell<T>>` 
so it can be mutated by multiple entities (in this case, both `rustc` and us).


```rust
# #![feature(rustc_private)]
# #![feature(box_syntax)]
# extern crate rustc;
# extern crate rustc_driver;
# extern crate getopts;
# extern crate syntax;
# use syntax::ast::Crate;
# use syntax::codemap::CodeMap;
# use rustc::session::Session;
# use rustc_driver::driver::{CompileController, CompileState};
# use rustc_driver::{CompilerCalls, Compilation};
# use std::rc::Rc;
# use std::cell::RefCell;
# #[derive(Debug, Clone, Default)]
# pub struct Metrics;
# fn analyse_ast(_ast: &Crate, _codemap: &CodeMap) -> Metrics {unimplemented!()}
#[derive(Default, Debug, Clone)]
pub struct Calls {
    pub unsafe_metrics: Rc<RefCell<Option<Metrics>>>,
}

impl<'a> CompilerCalls<'a> for Calls {
    fn build_controller(&mut self, 
                        _: &Session, 
                        _: &getopts::Matches) -> CompileController<'a> {
        let mut controller = CompileController::basic();
        controller.after_parse.stop = Compilation::Stop;

        let metrics = self.unsafe_metrics.clone();

        controller.after_parse.callback = box move |compile_state: &mut CompileState| {
            let ast = compile_state.krate.as_ref().unwrap();

            *metrics.borrow_mut() = Some(analyse_ast(
                ast,
                compile_state.session.codemap(),
            ));
        };

        controller
    }
}
# fn main() {}
```

You can see that after the AST has been analysed we set `metrics` (a pointer 
to the `unsafe_metrics` property inside `Calls`) to be the result of the 
analysis.

The helper function itself, `analyse_ast()`, then just creates a visitor, 
makes it visit the provided AST, then turns the result into a `Metrics` for 
our analyser.

```rust
# #![feature(rustc_private)]
# #![allow(dead_code)]
# extern crate syntax;
# extern crate syntax_pos;
# use syntax::codemap::CodeMap;
# use syntax::ast::Crate;
# use syntax::ext::quote::rt::Span;
# use syntax_pos::Loc;
# use syntax::visit::{self, Visitor};
fn analyse_ast(ast: &Crate, codemap: &CodeMap) -> Metrics {
    let mut visitor = UnsafeVisitor::new();

    // analyse the crate
    visit::walk_crate(&mut visitor, ast);

    // then resolve spans to line numbers and locations
    let spans = visitor
        .unsafe_lines
        .iter()
        .map(|span| {
            let start = Location::from(codemap.lookup_char_pos(span.lo));
            let end = Location::from(codemap.lookup_char_pos(span.hi));
            let diff = end.line - start.line;

            Row {
                start: start,
                end: end,
                num_lines: if diff == 0 { 1 } else { diff },
            }
        })
        .collect();

    Metrics {
        spans: spans,
        total_lines: codemap.count_lines(),
    }
}
# fn main() {}
# #[derive(Default, Debug)]
# struct UnsafeVisitor { unsafe_lines: Vec<Span> }
# impl<'a> Visitor<'a> for UnsafeVisitor {}
# impl UnsafeVisitor { fn new() -> UnsafeVisitor { UnsafeVisitor::default() }}
# struct Metrics { spans: Vec<Row>, total_lines: usize }
# struct Row { start: Location, end: Location, num_lines: usize }
# struct Location { line: usize }
# impl From<Loc> for Location { fn from(_other: Loc) -> Self { unimplemented!() } }
```


[Crate]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ast/struct.Crate.html
[Block]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ast/struct.Block.html
[Item]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ast/struct.Item.html
[Visitor]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/visit/trait.Visitor.html
[Span]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ext/quote/rt/struct.Span.html
[cap]: https://github.com/rust-lang/rust/blob/e3245948445b77c25cd9f3b29cbad3187aee3eb7/src/librustc_driver/driver.rs#L324