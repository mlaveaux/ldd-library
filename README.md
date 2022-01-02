# Description

A library to create and manipulate so called list decision diagrams, also abbreviated as LDDs, that is implemented in Rust. List decision diagrams are data structures that can efficiently represent sets of equal length vectors over natural numbers. For these data structures we can also efficiently implement standard set operations as well as several specialised operations for the purpose of graph exploration. For detailed documentation use `cargo doc --open` and visit the page for the `ldd` crate, which contains the library implementation. It has been tested, using `cargo test`, with rustc version 1.57.0.

The examples directory contains a reachability implementation to showcase the usage of this library.

# Related Work

This library is fully inspired by the work on [Sylvan](https://github.com/trolando/sylvan), which is a fully featured parallel implementation of an LDD (and BDD) library implemented in C.