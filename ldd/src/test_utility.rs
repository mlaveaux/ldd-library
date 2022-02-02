use crate::{Ldd, Storage, operations::*, Value};

use std::collections::HashSet;
use rand::Rng;

///! Functions in this module are only relevant for testing purposes.

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_vector(length: usize, max_value: Value) -> Vec<Value> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<Value> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(0..max_value));
    }

    vector
}

/// Returns a sorted vector of the given length with unique u64 values (from 0..max_value).
pub fn random_sorted_vector(length: usize, max_value: Value) -> Vec<Value> 
{
    use rand::prelude::IteratorRandom;

    let mut rng = rand::thread_rng(); 
    let mut result = (0..max_value).choose_multiple(&mut rng, length as usize);
    result.sort();
    result
}

/// Returns a set of 'amount' vectors where every vector has the given length.
pub fn random_vector_set(amount: usize, length: usize, max_value: Value) ->  HashSet<Vec<Value>>
{
    let mut result: HashSet<Vec<Value>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length, max_value));
    }

    result
}

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(storage: &mut Storage, iter: I) -> Ldd
    where I: Iterator<Item = &'a Vec<Value>>
{
    let mut result = storage.empty_set().clone();

    for vector in iter
    {
        let single = singleton(storage, vector);
        result = union(storage, result.borrow(), single.borrow());
    }

    result
}

/// Returns project(vector, proj), see [project]. Requires proj to be sorted.
pub fn project_vector(vector: &[Value], proj: &[Value]) -> Vec<Value>
{
    let mut result = Vec::<Value>::new();
    for i in proj
    {
        result.push(vector[*i as usize]);
    }
    result
}