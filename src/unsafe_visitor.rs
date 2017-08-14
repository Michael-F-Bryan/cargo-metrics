use syntax::codemap::CodeMap;
use syntax::ast::{Block, BlockCheckMode, Crate, Item, ItemKind, Mac, Unsafety};
use syntax::visit::{self, Visitor};
use syntax_pos::Loc;
use syntax::ext::quote::rt::Span;


pub fn analyse_ast(ast: &Crate, codemap: &CodeMap) -> Metrics {
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

#[derive(Debug, Clone, Default)]
struct UnsafeVisitor {
    pub unsafe_lines: Vec<Span>,
}

impl UnsafeVisitor {
    fn new() -> UnsafeVisitor {
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

impl From<Loc> for Location {
    fn from(other: Loc) -> Location {
        Location {
            filename: other.file.name.clone(),
            line: other.line,
            col: other.col.0,
        }
    }
}