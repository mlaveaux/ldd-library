use crate::{Ldd, Storage, Data, iterators::*};

use std::cmp::{self, Ordering};

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

/// Computes a meta LDD that is suitable for the [project] function from the
/// given projection indices. 
///
/// This function is useful to be able to cache the projection LDD instead of
/// computing it from the projection array every time.
pub fn compute_proj(storage: &mut Storage, proj: &[u64]) -> Ldd
{
    // Compute length of proj.
    let length = match proj.iter().max()
    {
        Some(x) => *x+1,
        None => 0
    };

    // Convert projection vectors to meta information.
    let mut result: Vec<u64> = Vec::new();
    for i in 0..length
    {
        let included = proj.contains(&i);

        if included {
            result.push(1);
        }
        else {
            result.push(0);
        }
    }
    
    singleton(storage, &result)
}

/// Computes the set of vectors projected onto the given indices, 
/// 
/// Let 'proj' be equal to compute_proj([i_0, ..., i_k]).
/// 
/// Formally, for a single vector <x_0, ..., x_n> we have that:
///     - project(<x_0, ..., x_n>, i_0 < ... < i_k) = <x_(i_0), ..., x_(i_k)>
///     - project(X, i_0 < ... < i_k) = { project(x, i_0 < ... < i_k) | x in X }. 
/// 
/// Note that the indices are sorted in the definition, but compute_proj
/// can take any array and ignores both duplicates and order. Also, it 
/// follows that i_k must be smaller than or equal to n as x_(i_k) is not 
/// defined otherwise.
pub fn project(storage: &mut Storage, set: &Ldd, proj: &Ldd) -> Ldd
{
    assert_ne!(proj, storage.empty_set(), "proj must be a singleton");

    if set == storage.empty_set()  {
        storage.empty_set().clone()
    } else if proj == storage.empty_vector() {
        // If meta is not defined then the rest is not in the projection (proj is always zero)
        storage.empty_vector().clone()
    } else {
        assert_ne!(set, storage.empty_vector(), "proj can be at most as high as set");

        let Data(proj_value, proj_down, _) = storage.get(proj);
        let Data(value, down, right) =  storage.get(set);

        match proj_value {
            0 => {
                let right_result = project(storage, &right, proj);
                let down_result = project(storage, &down, &proj_down);
                union(storage, &right_result, &down_result)
            }
            1 => {
                let right_result = project(storage, &right, proj);
                let down_result = project(storage, &down, &proj_down);
                if down_result == *storage.empty_set()
                {
                    right_result
                } 
                else 
                {
                    storage.insert(value, &down_result, &right_result)
                }
            }
            x => {
                panic!("proj has unexpected value {}", x);
            }
        }
    }
}

/// Computes a 'meta' LDD from the given read and write projections that is suitable for the relational_product.
/// 
/// The read and write projections are arrays of indices that are read, respectively written, by the corresponding sparse relation.
/// 
/// see [relational_product] for more information.
pub fn compute_meta(storage: &mut Storage, read_proj: &[u64], write_proj: &[u64]) -> Ldd
{
    // Compute length of meta.
    let length = cmp::max(
        match read_proj.iter().max()
        {
            Some(x) => *x,
            None => 0
        }
        , match write_proj.iter().max()
        {
            Some(x) => *x,
            None => 0
        });

    // Convert projection vectors to meta.
    let mut meta: Vec<u64> = Vec::new();
    for i in 0..length
    {
        let read = read_proj.contains(&i);
        let write = write_proj.contains(&i);

        if read && write {
            meta.push(3);
        }
        else if read {
            meta.push(1);
        }
        else if write {
            meta.push(2);
        }
        else {
            meta.push(0);
        }
    }

    singleton(storage, &meta)
}

/// Computes the set of vectors reachable in one step from the given 'set' as defined by the sparse relation rel, where meta = compute_meta(read_proj, write_proj).
/// 
/// relational_product(R, S, read_proj, write_proj) = { S[write_proj := y'] | project(x, read_proj) = x' and (x', y') in R }
///     where R is the relation and S the set.
///  
/// meta is a singleton vector where the value indicates the following for that index:
///     - 0 = not part the relation.
///     - 1 = only in read_proj
///     - 2 = only in write_proj
///     - 3 = both in read_proj and write_proj
pub fn relational_product(storage: &mut Storage, set: &Ldd, rel: &Ldd, meta: &Ldd) -> Ldd
{
    assert_ne!(set, storage.empty_vector());
    assert_ne!(rel, storage.empty_vector());
    assert_ne!(meta, storage.empty_set());

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
                if down_result == *storage.empty_set()
                {
                    right_result
                } 
                else 
                {
                    storage.insert(value, &down_result, &right_result)
                }
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
                        relational_product(storage, &set, &rel_right, meta)
                    }
                }
            }
            2 => {
                // Read the values present in the relation and continue with these values in the set.
                let Data(rel_value, rel_down, rel_right) = storage.get(rel);

                
                /*storage.insert(rel_value, &down_result, &right_result)
                
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
                }*/
                storage.empty_set().clone() 
            }
            3 => {
                storage.empty_set().clone() 
            }
    
            x => {
                panic!("meta has unexpected value: {}", x);
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
    if vector.is_empty() {
        *ldd == *storage.empty_vector()
    } else if *ldd == *storage.empty_vector() {
        false
    } else {
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
    } else if set == storage.empty_vector() {
        1
    } else {
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

    use std::collections::HashSet;
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
    }

    // Test the relational product function with random inputs.
    #[test]
    fn random_relational_product()
    {
        let mut storage = Storage::new();
        
        let set = random_vector_set(32, 10, 10);

        // First, test with a read only projection.
        {
            let ldd =  from_iter(&mut storage, set.iter());

            let proj: Vec<u64> = vec![0,3,7,9];
            let proj_ldd = compute_proj(&mut storage, &proj);
            let meta = compute_meta(&mut storage, &proj, &[]);

            let relation = project(&mut storage, &ldd, &proj_ldd);
            let result = relational_product(&mut storage, &ldd, &relation, &meta);
            let read_project = project(&mut storage, &result, &proj_ldd);

            assert_eq!(read_project, relation);
        }
    }

    // Test the project function with random inputs.
    #[test]
    fn random_project()
    {
        let mut storage = Storage::new();
        
        let set = random_vector_set(32, 10, 10);
        let proj: Vec<u64> = vec![0,3,7,9];

        // Compute a naive projection on the vector set.
        let mut expected_result: HashSet<Vec<u64>> = HashSet::new();
        for element in &set
        {
            let mut projection = Vec::<u64>::new();
            for i in &proj
            {
                projection.push(element[*i as usize]);
            }
            expected_result.insert(projection);
        }

        let ldd = from_iter(&mut storage, set.iter());
        let proj_ldd = compute_proj(&mut storage, &proj);
        let result = project(&mut storage, &ldd, &proj_ldd);

        let expected = from_iter(&mut storage, expected_result.iter());
        assert_eq!(result, expected);
    }
}
