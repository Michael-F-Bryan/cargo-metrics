# Instrumenting `rustc_driver`

There are a couple ways to influence the compilation process, but by far the 
most useful is with a `CompileController`. This is a fairly basic struct 
containing a bunch of `PhaseController`s which get invoked at particular 
points in the compilation process.

A `PhaseController` just contains a callback which is invoked with the 
current `CompileState`, and a `stop` attribute which says whether to abort
compilation.

From the `rust-lang/rust` repo's root directory they are defined in 
`src/librustc_driver/driver.rs`.

```rust
# #![feature(rustc_private)]
# extern crate rustc_driver;
# extern crate rustc_resolve;
# use rustc_driver::Compilation;
# use rustc_driver::driver::CompileState;
# use rustc_resolve::MakeGlobMap;
pub struct CompileController<'a> {
    pub after_parse: PhaseController<'a>,
    pub after_expand: PhaseController<'a>,
    pub after_hir_lowering: PhaseController<'a>,
    pub after_analysis: PhaseController<'a>,
    pub after_llvm: PhaseController<'a>,
    pub compilation_done: PhaseController<'a>,

    pub make_glob_map: MakeGlobMap,
    // Whether the compiler should keep the ast beyond parsing.
    pub keep_ast: bool,
    // -Zcontinue-parse-after-error
    pub continue_parse_after_error: bool,
}

pub struct PhaseController<'a> {
    pub stop: Compilation,
    // If true then the compiler will try to run the callback even if the phase
    // ends with an error. Note that this is not always possible.
    pub run_callback_on_error: bool,
    pub callback: Box<Fn(&mut CompileState) + 'a>,
}
# fn main(){}
```

The `CompileState` is a dumb object which just wraps the various bits of state
required for the compilation process up into a single struct. All of these bits
and pieces are declared `pub` so they can be accessed directly by your 
`PhaseController`. Its full definition is in `src/librustc_driver/driver.rs`, 
however it should look something like this (valid as of *2017-08-14*).

```rust
# #![feature(rustc_private)]
# extern crate arena;
# extern crate rustc;
# extern crate rustc_driver;
# extern crate rustc_plugin;
# extern crate rustc_metadata;
# extern crate syntax;
use arena::DroplessArena;
use syntax::ast;
use rustc::hir::{self, map as hir_map};
use rustc::ty::{self, GlobalArenas, Resolutions, TyCtxt};
use rustc::session::Session;
use rustc::session::config::{Input, OutputFilenames};
use rustc_metadata::cstore::CStore;
use rustc_plugin::registry::Registry;
use std::path::Path;

pub struct CompileState<'a, 'tcx: 'a> {
    pub input: &'a Input,
    pub session: &'tcx Session,
    pub krate: Option<ast::Crate>,
    pub registry: Option<Registry<'a>>,
    pub cstore: Option<&'a CStore>,
    pub crate_name: Option<&'a str>,
    pub output_filenames: Option<&'a OutputFilenames>,
    pub out_dir: Option<&'a Path>,
    pub out_file: Option<&'a Path>,
    pub arena: Option<&'tcx DroplessArena>,
    pub arenas: Option<&'tcx GlobalArenas<'tcx>>,
    pub expanded_crate: Option<&'a ast::Crate>,
    pub hir_crate: Option<&'a hir::Crate>,
    pub hir_map: Option<&'a hir_map::Map<'tcx>>,
    pub resolutions: Option<&'a Resolutions>,
    pub analysis: Option<&'a ty::CrateAnalysis>,
    pub tcx: Option<TyCtxt<'a, 'tcx, 'tcx>>,
    #[cfg(feature="llvm")]
    pub trans: Option<&'a trans::CrateTranslation>,
}
# fn main(){}
```


## Making `rustc_driver` Run our `CompileController`

As `@nrc` mentions in [stupid-stats]:

> There are two primary ways to customise compilation - high level control of
the driver using `CompilerCalls` and controlling each phase of compilation
using a `CompileController`. The former lets you customise handling of command
line arguments etc., the latter lets you stop compilation early or execute
code between phases.

We are mainly interested in the `CompileController` because that's where all 
the action is, but to let us inject one into `rustc_driver` we'll need to 
define our own type which implements `CompilerCalls`. Most of the methods for
`CompilerCalls` have sane defaults, so we can get away with only implementing
`build_controller()`.


```rust
# #![feature(rustc_private)]
# #![allow(dead_code)]
# extern crate rustc_driver;
# extern crate getopts;
# extern crate rustc;
# use rustc_driver::CompilerCalls;
# use rustc_driver::driver::CompileController;
# use rustc::session::Session;
struct Calls;

impl<'a> CompilerCalls<'a> for Calls {
    fn build_controller(&mut self, 
                        _: &Session, 
                        _: &getopts::Matches) -> CompileController<'a> {
        panic!("TODO: construct a CompileController")
    }
}
# fn main() {}
```

We'll flesh out the `build_controller()` method later on when we start going
into concrete examples, but for now lets just try to get something to run.

The main entrypoint for `rustc_driver` is the `run_compiler()` function. This 
takes a list of command line arguments, a mutable reference to your 
`CompilerCalls` implementation, and a couple other optional fields. Note the 
mutable reference bit, the `CompilerCalls`'s `'a` lifetime allows our various
phases to manipulate the `Calls` which was passed in so we can pass information
from `rustc` to the caller.

```rust,no_run
# #![feature(rustc_private)]
# extern crate rustc_driver;
# use rustc_driver::RustcDefaultCalls as Calls;
use std::env;

fn main() {
    let mut calls = Calls;
    let args: Vec<String> = env::args().collect();
    let (compile_result, _session) = rustc_driver::run_compiler(&args, 
                                                                &mut calls, 
                                                                None, 
                                                                None);
    if let Err(e) = compile_result {
        panic!("Compilation failed! {:?}", e);
    }
}
```

If we create a new project with `cargo` and put the above code in its `main.rs`
you should see something like this when it's run:

```bash
$ cargo run -- src/lib.rs 
    ...
thread 'main' panicked at 'TODO: construct a CompileController', src/lib.rs:18:8
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```

> **Note:** If you just do `cargo run`, it'll print the usual `rustc` help 
> text. This is because of the default impl for `CompilerCalls::no_input()`. 
> Check out the source code for more.

Now we know we can inject code into `rustc_driver` we can move onto the the 
first example, an analysis pass which counts the number of unsafe lines of 
code in a crate.


[stupid-stats]: https://github.com/nrc/stupid-stats#the-driver-customisation-apis