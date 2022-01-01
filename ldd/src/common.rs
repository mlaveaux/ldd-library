#[cfg(test)]
use crate::{Ldd, Storage, operations::*};

#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
use rand::Rng;

// These functions are only relevant for testing purposes.

/// Returns a vector of the given length with random u64 values.
/// We only generate values within 0 to 10 for testing purposes.
#[cfg(test)]
pub fn random_vector(length: u64) -> Vec<u64> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<u64> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(0..10));
    }

    vector
}

/// Returns a set of 'amount' vectors where every vector has the given length.
#[cfg(test)]
pub fn random_vector_set(amount: u64, length: u64) ->  HashSet<Vec<u64>>
{
    let mut result: HashSet<Vec<u64>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length));
    }

    result
}

/// Returns an LDD containing all elements of the given iterator over vectors.
#[cfg(test)]
pub fn from_iter<'a, I>(storage: &mut Storage, iter: I) -> Ldd
    where I: Iterator<Item = &'a Vec<u64>>
{
    let mut result = storage.empty_set().clone();

    for vector in iter
    {
        let single = singleton(storage, vector);
        result = union(storage, &result, &single);
    }

    result
}