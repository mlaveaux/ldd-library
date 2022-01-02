//! # LDD
//!
//! A library to create and manipulate so called list decision diagrams, also
//! abbreviated as LDDs. List decision diagrams are data structures that can
//! efficiently represent sets of vectors over natural numbers \[Dijk18\]. 

//! # Representation
//!
//! An LDD is inductively defined as follows in the literature 
//! 
//! > n :: node(value, n, n) | true | false
//! 
//! Given an LDD n then \[n\] is defined as:
//!   - \[false\] = empty set
//!   - \[true\] = { <> }
//!   - \[node(value, down, right)\] = { value x | x in \[down\] } union
//!     \[right\]
//! 
//! Where value is a positive number. Since 'true' and 'false' are not very
//! insightful and clash with Rust keywords we use 'empty vector' and 'empty
//! set' for the constans 'true' and 'false' respectively.
//!
//! # Citations
//! 
//! \[Dijk18\] ---

mod storage;
mod operations;
mod format;
mod iterators;
mod common;

pub use storage::*;
pub use operations::*;
pub use format::*;