# An Unofficial Guide to Using Rustc

Many people familiar with C and C++ will have heard of [libclang]. This is a
really powerful library which gives you access to the internals of the `clang`
compiler so you can analyse a C/C++ program, or even rewrite bits of it on the
fly. This allows you to write tools for detecting undefined behaviour or 
insert additional statements to help calculate code coverage.

The Rust compiler gives you hooks for analysing or manipulating a program 
during the compilation process. This is arguably more powerful because you 
have access to *all* of the compiler internals... With the trade-off being
you can only access it from *nightly* given the perpetually unstable nature of
these internals (`rustc` and the Rust language are continually evolving after 
all).

This tutorial should hopefully give you a better understanding of the 
compilation process and how you can plug into `rustc` to do magical and 
wonderous things. It's written from the point-of-view of someone wanting to 
write a generic Rust crate analyser as a `cargo` subcommand (`cargo metrics`).


## Useful Links and Prior Art

Often the easiest way to get started is to look at existing uses of `rustc` 
internals in the wild and adapt what they've done to your use case.

Here are several repositories which already use `rustc` for analysing Rust 
code.

- [Stupid Stats (highly recommended - I actually based this tutorial on `nrc`'s work)](https://github.com/nrc/stupid-stats)
- [Clippy](https://github.com/rust-lang-nursery/rust-clippy)
- [rust-semverver](https://github.com/ibabushkin/rust-semverver)
- [miri](https://github.com/solson/miri)
- [cargo-doc-coverage](https://gitlab.com/integer32llc/cargo-doc-coverage)


## Initial Setup

You don't need anything special to start using `rustc`'s internals, just 
switch to `nightly` and you should be able to follow along.

```bash
$ rustup default nightly
```

While it's not required, it's quite useful to have a copy of the `rustdoc`
docs for `rustc` (and its various other crates) handy, as well as a local
checkout of the [rust-lang/rust] repo. I've set up a cron job on my laptop to
periodically regenerate the internal compiler documentation and push them to
[GitHub Pages][ghp] to make your life easier.


[libclang]: https://clang.llvm.org/docs/Tooling.html
[ghp]: https://michael-f-bryan.github.io/rustc-internal-docs/rustc/index.html
[rust-lang/rust]: https://github.com/rust-lang/rust