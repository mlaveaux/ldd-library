// List Decision Diagrams, abbreviated LDD, are data structures that represent sets of fixed length vectors. 
// An LDD represents a set as follows. Given an LDD n then [[n]] is defined as:
// [[false]] = emptyset
// [[true]] = { <> } (the singleton set containing the empty vector)
// [[node(value, down, right)]] = { value x | x in [[down]] } union [[right]] (where value x is the concatenation of vectors)

// Every LDD points to its root node by means of an index.
pub type Ldd = usize;

mod storage;
mod operations;
mod format;
pub use storage::*;
pub use operations::*;
pub use format::*;

mod tests;