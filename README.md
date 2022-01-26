# Description

A library to create and manipulate so called list decision diagrams, also abbreviated as LDDs, that is implemented in Rust. List decision diagrams are data structures that can efficiently represent sets of equal length vectors over natural numbers. For these data structures we can also efficiently implement standard set operations as well as several specialised operations for the purpose of graph exploration. For detailed documentation use `cargo doc --open` and visit the page for the `ldd` crate, which contains the library implementation. Compiling the library requires at least rustc version 1.58.0 and we use 2021 edition rust.

# Tests

Tests can be performed using `cargo test`.

# Examples

The examples directory contains a reachability tool called `reach` to showcase the usage of this library. This tool can read the `.ldd` files in the format of the [Sylvan](https://github.com/trolando/sylvan) library. The tool can be executed using for example `cargo run --release ./examples/reach/models/anderson.4.ldd`.

# Benchmarks

For benchmarks we use [Criterion.rs](https://crates.io/crates/criterion) and these benchmarks can be executed using `cargo bench`. For more functionality, such as history reports instead of only comparing to the previous benchmark run, we can install `cargo-criterion` using `cargo install criterion` and then run the benchmarks using `cargo criterion`.

Furthermore, the `reach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/reach` can be profiled using any standard executable profiler.

# Related Work

This library is fully inspired by the work on [Sylvan](https://github.com/trolando/sylvan), which is a fully featured parallel implementation of an LDD (and BDD) library implemented in C.