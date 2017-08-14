use syntax::ast::{Block, BlockCheckMode, Item, ItemKind, Mac, Unsafety};
use syntax::visit::{self, Visitor};
use syntax_pos::Loc;
use syntax::ext::quote::rt::Span;


#[derive(Debug, Clone, Default)]
pub struct UnsafeVisitor {
    pub unsafe_lines: Vec<Span>,
}

impl UnsafeVisitor {
    pub fn new() -> UnsafeVisitor {
        UnsafeVisitor::default()
    }
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
            _ => {}
        }

        visit::walk_item(self, item);
    }

    fn visit_block(&mut self, block: &'a Block) {
        if let BlockCheckMode::Unsafe(_) = block.rules {
            self.unsafe_lines.push(block.span.clone());
        }

        visit::walk_block(self, block);
    }

    fn visit_mac(&mut self, _: &'a Mac) {}
}


#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub spans: Vec<(Location, Location, usize)>,
}

#[derive(Debug, Clone, Default)]
pub struct Location {
    pub filename: String,
    pub line: usize,
    pub col: usize,
}

impl From<Loc> for Location {
    fn from(other: Loc) -> Location {
        Location {
            filename: other.file.name.clone(),
            line: other.line,
            col: other.col.0,
        }
    }
}