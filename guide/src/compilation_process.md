# The Compilation Process

While the compilation process itself is quite complex and involves many moving
parts, it can be broken down into a handful of discrete phases. These are:

- Lexing and Parsing 
- Lowering to HIR
- Analysis (type checking, inference, `borrowck`, etc)
- Translation to LLVM IR
- Optimisation and code generation (done by LLVM)
- Linking into a single executable (also outsourced, usually to something like `ld`)

All of these steps are coordinated by the `driver` which resides in 
[`rustc_driver`]. A lot of this tutorial will be interacting with 
`rustc_driver` and the various knobs and hooks it gives us access to.


[`rustc_driver`]: https://github.com/rust-lang/rust/tree/master/src/librustc_driver