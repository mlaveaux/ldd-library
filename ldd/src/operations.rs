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

/// Computes the set of vectors reachable in one step from the set by the given sparse relation.
/// 
/// `{u -> v | u in set}` where `->` is described by rel and meta.
/// 
/// meta is a singleton vector where the value indicates the following:
///     - 0 = not part the relation.
pub fn relational_product(storage: &mut Storage, set: &Ldd, rel: &Ldd, meta: &Ldd) -> Ldd
{
    if set == storage.empty_set() || rel == storage.empty_set() {
        storage.empty_set().clone()
    } else if meta == storage.empty_vector() {
        // If meta is not defined then the rest is not in the relation (meta is always zero)
        set.clone()
    } else {
        let Data(meta_value, meta_down, _) = storage.get(meta);

        let result = match meta_value
        {
            0 => {
                // Consider all values on this level part of the output and continue with rest.
                let Data(value, down, right) = storage.get(set);

                let right_result = relational_product(storage, &right, rel, meta);
                let down_result = relational_product(storage, &down, rel, &meta_down);

                storage.insert(value, &down_result, &right_result)
            }
            1 => {
                // Read the values present in the relation and continue with these values in the set.
                let Data(set_value, set_down, set_right) = storage.get(set);
                let Data(rel_value, rel_down, rel_right) = storage.get(rel);
                
                match set_value.cmp(&rel_value) {
                    Ordering::Less => {
                        relational_product(storage, &set_right, rel, meta)                        
                    }                    
                    Ordering::Equal => {
                        let down_result = relational_product(storage, &set_down, &rel_down, &meta_down);
                        let right_result = relational_product(storage, &set_right, &rel_right, meta);
                        if down_result == *storage.empty_set()
                        {
                            right_result
                        } 
                        else 
                        {
                            storage.insert(set_value, &down_result, &right_result)
                        }  
                    }
                    Ordering::Greater => {
                        storage.empty_set().clone()                        
                    }
                }
            }
    
            _ => {
                panic!("meta has unexpected value");
            }
        };

        result
    }
}

/// Returns the largest subset of 'a' that does not contains elements of 'b', i.e., set difference.
pub fn minus(storage: &mut Storage, a: &Ldd, b: &Ldd) -> Ldd
{
    if a == b || a == storage.empty_set() {
        storage.empty_set().clone()
    } else if b == storage.empty_set() {
        a.clone()
    } else {
        let Data(a_value, a_down, a_right) = storage.get(a);
        let Data(b_value, b_down, b_right) = storage.get(b);

        match a_value.cmp(&b_value) {
            Ordering::Less => {
                let right_result = minus(storage, &a_right, b);
                storage.insert(a_value, &a_down, &right_result)
            },
            Ordering::Equal => {
                let down_result = minus(storage, &a_down, &b_down);
                let right_result = minus(storage, &a_right, &b_right);
                if down_result == *storage.empty_set()
                {
                    right_result
                } 
                else 
                {
                    storage.insert(a_value, &down_result, &right_result)
                }                
            },
            Ordering::Greater => {
                minus(storage, a, &b_right)
            }
        }
    }
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
        let Data(a_value, a_down, a_right) = storage.get(a);
        let Data(b_value, b_down, b_right) = storage.get(b);

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
    if vector.is_empty()
    {
        *ldd == *storage.empty_vector()
    }
    else if *ldd == *storage.empty_vector()
    {
        false
    }
    else
    {
        for Data(value, down, _) in iter_right(storage, ldd)
        {            
            match value.cmp(&vector[0]) { 
                Ordering::Less => { continue; } 
                Ordering::Equal => { return element_of(storage, &vector[1..], &down); }
                Ordering::Greater => { return false; }
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
        for Data(_, down, _) in iter_right(storage, set)
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

    use std::ops::Sub;
    use rand::Rng;

    // Compare the LDD element_of implementation for random inputs.
    #[test]
    fn random_element_of()
    {    
        let mut storage = Storage::new();
        let mut rng = rand::thread_rng();  

        let length = 10;
        let set = random_vector_set(32, length, 10);
        let ldd = from_iter(&mut storage, set.iter());
        
        // All elements in the set should be contained in the ldd.
        for expected in &set
        {
            assert!(element_of(&storage, &expected, &ldd));
        }

        // No shorter vectors should be contained in the ldd (try several times).
        for _ in 0..10
        {
            let short_vector = random_vector(rng.gen_range(0..length), 10);
            assert!(!element_of(&storage, &short_vector, &ldd));
        }

        // No longer vectors should be contained in the ldd.
        for _ in 0..10
        {
            let short_vector = random_vector(rng.gen_range(length+1..20), 10);
            assert!(!element_of(&storage, &short_vector, &ldd));
        }

        // Try vectors of correct size with both the set and ldd.
        for _ in 0..10
        {
            let vector = random_vector(length, 10);
            assert_eq!(set.contains(&vector), element_of(&storage, &vector, &ldd));
        }
    }

    // Compare the HashSet implementation of union with the LDD union implementation for random inputs.
    #[test]
    fn random_union()
    {
        let mut storage = Storage::new();

        let set_a = random_vector_set(32, 10, 10);
        let set_b = random_vector_set(32, 10, 10);

        let a = from_iter(&mut storage, set_a.iter());
        let b = from_iter(&mut storage, set_b.iter());
        let result = union(&mut storage, &a, &b);

        let expected = from_iter(&mut storage, set_a.union(&set_b));
        assert_eq!(result, expected);
    }
    
    // Compare the singleton implementation with a random vector used as input.
    #[test]
    fn random_singleton()
    {
        let mut storage = Storage::new();
        let vector = random_vector(10, 10);

        let ldd = singleton(&mut storage, &vector[..]);

        // Check that ldd contains exactly vector that is equal to the vector.
        let mut it = iter(&storage, &ldd);
        let result = it.next().unwrap();
        assert_eq!(vector, result);
        assert_eq!(it.next(), None); // No other vectors.
    }

    // Test the len function with random inputs.
    #[test]
    fn random_len()
    {
        let mut storage = Storage::new();

        let set = random_vector_set(32, 10, 10);
        let ldd = from_iter(&mut storage, set.iter());

        assert_eq!(set.len(), len(&storage, &ldd));
    }

    // Test the minus function with random inputs.
    #[test]
    fn random_minus()
    {
        let mut storage = Storage::new();
        
        let set_a = random_vector_set(32, 10, 10);
        let set_b = {
            let mut result = random_vector_set(32, 10, 10);

            // To ensure some overlap (which is unlikely) we insert some elements of a into b.
            let mut it = set_a.iter();
            for _ in 0..16
            {
                result.insert(it.next().unwrap().clone());
            }

            result
        };

        let expected_result = set_a.sub(&set_b);
        
        let a = from_iter(&mut storage, set_a.iter());
        let b = from_iter(&mut storage, set_b.iter());
        let result = minus(&mut storage, &a, &b);
        let expected = from_iter(&mut storage, expected_result.iter());
        assert_eq!(result, expected);
        
        for expected in expected_result.iter()
        {
            assert!(element_of(&storage, expected, &result));
        }

        for value in iter(&storage, &result)
        {
            assert!(expected_result.contains(&value));
        }
    }

}
