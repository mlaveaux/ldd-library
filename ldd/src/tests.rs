use crate::{Ldd, storage::Storage, operations::*, iterators::*};

use std::collections::HashSet;
use rand::Rng;

// Returns a vector of the given length with random u64 values.
fn random_vector(length: u64) -> Vec<u64> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<u64> = Vec::new();
    for _ in 1..length
    {
        vector.push(rng.gen());
    }

    vector
}

// Returns a set of 'amount' vectors where every vector has the given length.
fn random_vector_set(amount: u64, length: u64) ->  HashSet<Vec<u64>>
{
    let mut result: HashSet<Vec<u64>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 1..amount
    {
        result.insert(random_vector(length));
    }

    result
}

// Construct and Ldd from a given HashSet.
fn from_hashset(storage: &mut Storage, set: &HashSet<Vec<u64>>) -> Ldd
{
    let mut result = storage.empty_set();

    for vector in set
    {
        let single = singleton(storage, vector);
        result = union(storage, result, single);
    }

    result
}

// Compare the LDD element_of implementation for random inputs.
#[test]
fn random_element_of()
{    
    let mut storage = Storage::new();
    let mut rng = rand::thread_rng();  

    let length = 10;
    let set = random_vector_set(32, length);
    let ldd = from_hashset(&mut storage, &set);
    
    // All elements in the set should be contained in the ldd.
    for expected in &set
    {
        assert!(element_of(&storage, &expected, &ldd));
    }

    // No shorter vectors should be contained in the ldd (try several times).
    for _ in 1..10
    {
        let short_vector = random_vector(rng.gen_range(0..length));
        assert!(!element_of(&storage, &short_vector, &ldd));
    }

    // No longer vectors should be contained in the ldd.
    for _ in 1..10
    {
        let short_vector = random_vector(rng.gen_range(length+1..20));
        assert!(!element_of(&storage, &short_vector, &ldd));
    }

    // Try vectors of correct size with both the set and ldd.
    for _ in 1..10
    {
        let vector = random_vector(length);
        assert_eq!(set.contains(&vector), element_of(&storage, &vector, &ldd));
    }
}

// Compare the HashSet implementation of union with the LDD union implementation for random inputs.
#[test]
fn random_union()
{
    let mut storage = Storage::new();

    let set_a = random_vector_set(32, 10);
    let set_b = random_vector_set(32, 10);

    let a = from_hashset(&mut storage, &set_a);
    let b = from_hashset(&mut storage, &set_b);
    let result = union(&mut storage, a, b);

    for expected in set_a.union(&set_b)
    {
        assert!(element_of(&storage, &expected, &result));
    }

    for vector in iter(&storage, &result)
    {
        assert!(set_a.contains(&vector) || set_b.contains(&vector));
    }
}

// Test the iterator implementation.
#[test]
fn random_iter()
{
    let mut storage = Storage::new();

    let set = random_vector_set(32, 10);
    let ldd = from_hashset(&mut storage, &set);

    // Check that the number of iterations matches the number of elements in the set.
    assert!(iter(&storage, &ldd).count() == set.len());

    // Every iterated element must be in the set.
    for vector in iter(&storage, &ldd)
    {
        assert!(set.contains(&vector));
    }
}