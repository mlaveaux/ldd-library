#[cfg(test)]
use crate::{Ldd, Storage, operations::*};

#[cfg(test)]
use std::collections::HashSet;

#[cfg(test)]
use rand::Rng;

// These functions are only relevant for testing purposes.

/// Returns a vector of the given length with random u64 values (from 0..max_value).
#[cfg(test)]
pub fn random_vector(length: u64, max_value: u64) -> Vec<u64> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<u64> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(0..max_value));
    }

    vector
}

/// Returns a sorted vector of the given length with unique u64 values (from 0..max_value).
#[cfg(test)]
pub fn random_sorted_vector(length: u64, max_value: u64) -> Vec<u64> 
{
    use rand::prelude::IteratorRandom;

    let mut rng = rand::thread_rng(); 
    let mut result =(0..max_value).choose_multiple(&mut rng, length as usize);
    result.sort();
    result
}

/// Returns a set of 'amount' vectors where every vector has the given length.
#[cfg(test)]
pub fn random_vector_set(amount: u64, length: u64, max_value: u64) ->  HashSet<Vec<u64>>
{
    let mut result: HashSet<Vec<u64>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length, max_value));
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

/// Returns project(vector, proj), see [project]. Requires proj to be sorted.
#[cfg(test)]
pub fn project_vector(vector: &[u64], proj: &[u64]) -> Vec<u64>
{
    let mut result = Vec::<u64>::new();
    for i in proj
    {
        result.push(vector[*i as usize]);
    }
    result
}