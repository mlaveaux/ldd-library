 > ⚠️ **important** This repository is archived, and its development is continued in the [merc](https://github.com/MERCorg/merc) repository. In the future an LDD implementation will also added to the [OxiDD](https://oxidd.net/) project.

# Description

A library to create and manipulate so called list decision diagrams, also abbreviated as LDDs, that is implemented in Rust. List decision diagrams are data structures that can efficiently represent sets of equal length vectors over natural numbers. For these data structures we can also efficiently implement standard set operations as well as several specialised operations for the purpose of graph exploration. For detailed documentation use `cargo doc --open` and visit the page for the `ldd` crate, which contains the library implementation. Compiling the library requires at least rustc version 1.58.0 and we use 2021 edition rust.

# Tests

Tests can be performed using `cargo test`.

# Examples

The examples directory contains a reachability tool called `reach` to showcase the usage of this library. This tool can read the `.ldd` files in the format of the [Sylvan](https://github.com/trolando/sylvan) library. The tool can be executed using for example `cargo run --release examples/reach/models/anderson.4.ldd`.

# Benchmarks

For micro benchmarks we use [Criterion.rs](https://crates.io/crates/criterion) and these benchmarks can be executed using `cargo bench`. For more functionality, such as history reports instead of only comparing to the previous benchmark run, we can install `cargo-criterion` using `cargo install criterion` and then run the benchmarks using `cargo criterion`. Note that this latter option is still experimental and might not always work.

The `reach` tool can be compared to the `lddmc` tool provided in [Sylvan](https://github.com/trolando/sylvan) to benchmark actual graph exploration on the models provided in `examples/reach/models`. For this the tool is build in release configuration using `cargo build --release`. We pass the options `-sbfs -w1` to `lddmc` to disable multi-threading and use the same exploration strategy for equal comparison. We use `hyperfine` for the benchmarking process, which can be installed with `cargo install hyperfine`. The benchmarks are the averages of at least ten runs obtained using `hyperfine --warmup 3 <command>`. These benchmarks have been performed on a machine with an Intel(R) Core(TM) i7-7700HQ CPU with 32GB ram.

| Model                 | lddmc (1 worker) | lddmc (8 workers) | reach (2cd4a51) | reach (f18cb62) |
| ---                   | ---:             | ---:              | ---:            |  ---:
| anderson.4.ldd        |    0.19          |    0.79           |    0.14         |   0.10
| anderson.6.ldd        |    6.40          |    6.00           |    9.88         |   6.77
| anderson.8.ldd        |   52.06          |   33.20           |   83.86         |  61.13
| bakery.4.ldd          |    0.42          |    1.62           |    0.61         |   0.44
| bakery.5.ldd          |   12.04          |    9.87           |   21.56         |  15.27
| bakery.6.ldd          |   11.95          |    9.37           |   17.29         |  12.51
| bakery.7.ldd          |   45.46          |   28.54           |   91.05         |  66.10
| blocks.2.ldd          |    0.18          |    0.84           |    n/a*         |   n/a*
| blocks.3.ldd          |    3.46          |    7.20           |    n/a*         |   n/a*
| blocks.4.ldd          |  327.03          |  435.12           |    n/a*         |   n/a*
| collision.4.ldd       |   54.25          |   38.65           |  140.78         | 102.74
| collision.5.ldd       |  322.08          |  189.41           | 1048.82         | 545.88
| collision.6.ldd       | 4930.45          | 4040.28           |   OOM**         |  OOM**
| lifts.6.ldd           |    2.49          |    4.52           |    7.65         |   5.00
| lifts.7.ldd           |   20.03          |   20.11           |   46.70         |  31.23
| schedule_world.2.ldd  |    1.80          |    2.09           |    3.92         |   2.97
| schedule_world.2.ldd  |   63.91          |   36.29           |  175.95         | 138.47

\* For these benchmarks the `reach` tool indicates that the LDD was not valid; so that is most likely a bug. 
\*\* This benchmark requires a lot of memory and the ldd-library used 32 bytes per LDD node as opposed to 16 bytes in Sylvan. This has since been improved to 24 bytes per LDD node for the ldd-library.

# Profiling

The `reach` tool can be build using the `bench` compilation profile using `cargo build --profile bench` after which the resulting executable `target/release/reach` can be profiled using any standard executable profiler. This compilation profile contains debugging information to show where time is being spent, but the code is optimised the same as in a release configuration.

# Related Work

This library is fully inspired by the work on [Sylvan](https://github.com/trolando/sylvan), which is a fully featured parallel implementation of an LDD (and BDD) library implemented in C.
