# Counting Lines of Unsafe

The first metric we're going to calculate is the number of lines of unsafe
in a crate. For this we don't need to run any of the later steps like type
checking or `borrowck`, so we'll just use the AST generated after the compiler
finishes parsing. We can use the `CompileController.after_parse` to hook in
after the compiler has finished parsing and expanding macros.

For the actual AST inspection, the `syntax` crate provides an extremely handy
[`Visitor`] trait which will recursively visit each node in a [`Crate`]. So
that means creating an unsafe code counter will require:

- Creating a type which implements the [`Visitor`] trait, overriding just the
  methods which inspect `Block`s and `Item`s
- In the `Visitor` impl store the [`Span`] for all `unsafe` blocks we find
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


```rust
# #![feature(rustc_private)]
# extern crate rustc;
# extern crate rustc_driver;
# extern crate getopts;
# use rustc_driver::{CompilerCalls, Compilation};
# use rustc_driver::driver::CompileController;
# use rustc::session::Session;
pub struct Calls;

impl<'a> CompilerCalls<'a> for Calls {
    fn build_controller(&mut self, _: &Session, _: &getopts::Matches) -> CompileController<'a> {
        let mut controller = CompileController::basic();
        controller.after_parse.stop = Compilation::Stop;

        controller
    }
}
# fn main() {}
```


[`Crate`]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ast/struct.Crate.html
[`Visitor`]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/visit/trait.Visitor.html
[`Span`]: https://michael-f-bryan.github.io/rustc-internal-docs/syntax/ext/quote/rt/struct.Span.html