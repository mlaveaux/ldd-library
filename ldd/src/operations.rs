use crate::{Ldd, Storage, Data, iterators::*};

use std::cmp::Ordering;

/// Returns an LDD containing only the given vector, i.e., { vector }.
pub fn singleton(storage: &mut Storage, vector: &[u64]) -> Ldd
{
    let mut root = storage.empty_vector().clone();
    let empty_set = storage.empty_set().clone();
    for val in vector.iter().rev()
    {
        root = storage.insert(*val, &root, &empty_set);
    }

    root
}

/// Returns the union of the given LDDs.
pub fn union(storage: &mut Storage, a: &Ldd, b: &Ldd) -> Ldd
{
    if a == b {
        a.clone()
    } else if a == storage.empty_set() {
        b.clone()
    } else if b == storage.empty_set() {
        a.clone()
    } else {
        let Data(a_value, a_down, a_right) = storage.get(&a);
        let Data(b_value, b_down, b_right) = storage.get(&b);

        match a_value.cmp(&b_value) {
            Ordering::Less => {
                let result = union(storage, &a_right, b);
                storage.insert(a_value, &a_down, &result)
            },
            Ordering::Equal => {
                let down_result = union(storage, &a_down, &b_down);
                let right_result = union(storage, &a_right, &b_right);
                storage.insert(a_value, &down_result, &right_result)
            },
            Ordering::Greater => {
                let result = union(storage, a, &b_right);
                storage.insert(b_value, &b_down, &result)
            }
        }
    }
}

/// Returns true iff the set contains the vector.
pub fn element_of(storage: &Storage, vector: &[u64], ldd: &Ldd) -> bool
{
    if vector.len() == 0
    {
        *ldd == *storage.empty_vector()
    }
    else if *ldd == *storage.empty_vector()
    {
        false
    }
    else
    {
        for Data(value, down, _) in iter_right(&storage, ldd)
        {            
            if value == vector[0] {
                return element_of(storage, &vector[1..], &down);
            } else if value > vector[0] {
                return false;
            }
        }

        false
    }    
}

/// Returns the number of elements in the set.
pub fn len(storage: &Storage, set: &Ldd) -> usize
{
    if set == storage.empty_set() {
        0
    }
    else if set == storage.empty_vector() {
        1
    }
    else
    {
        let mut result: usize = 0;
        for Data(_, down, _) in iter_right(&storage, &set)
        {
            result += len(storage, &down);
        }

        result
    }
}

#[cfg(test)]
mod tests
{
    use super::*;    
    use crate::common::*;
    use rand::Rng;

    // Compare the LDD element_of implementation for random inputs.
    #[test]
    fn random_element_of()
    {    
        let mut storage = Storage::new();
        let mut rng = rand::thread_rng();  

        let length = 10;
        let set = random_vector_set(32, length);
        let ldd = from_iter(&mut storage, set.iter());
        
        // All elements in the set should be contained in the ldd.
        for expected in &set
        {
            assert!(element_of(&storage, &expected, &ldd));
        }

        // No shorter vectors should be contained in the ldd (try several times).
        for _ in 0..10
        {
            let short_vector = random_vector(rng.gen_range(0..length));
            assert!(!element_of(&storage, &short_vector, &ldd));
        }

        // No longer vectors should be contained in the ldd.
        for _ in 0..10
        {
            let short_vector = random_vector(rng.gen_range(length+1..20));
            assert!(!element_of(&storage, &short_vector, &ldd));
        }

        // Try vectors of correct size with both the set and ldd.
        for _ in 0..10
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

        let a = from_iter(&mut storage, set_a.iter());
        let b = from_iter(&mut storage, set_b.iter());
        let result = union(&mut storage, &a, &b);

        for expected in set_a.union(&set_b)
        {
            assert!(element_of(&storage, &expected, &result));
        }

        for vector in iter(&storage, &result)
        {
            assert!(set_a.contains(&vector) || set_b.contains(&vector));
        }
    }
    
    // Compare the singleton implementation with a random vector used as input.
    #[test]
    fn random_singleton()
    {
        let mut storage = Storage::new();
        let vector = random_vector(10);

        let ldd = singleton(&mut storage, &vector[..]);

        // Check that ldd contains exactly vector that is equal to the vector.
        let result = iter(&storage, &ldd).next().unwrap();
        assert_eq!(vector, result);
    }

    // Test the len function with random inputs.
    #[test]
    fn random_len()
    {
        let mut storage = Storage::new();

        let set = random_vector_set(32, 10);
        let ldd = from_iter(&mut storage, set.iter());

        assert_eq!(set.len(), len(&storage, &ldd));
    }
}
