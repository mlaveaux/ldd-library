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

    if proj == storage.empty_vector() {
        // If meta is not defined then the rest is not in the projection (proj is always zero)
        storage.empty_vector().clone()
    } else if set == storage.empty_set() {
        storage.empty_set().clone()
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

/// Computes a meta LDD from the given read and write projections that is
/// suitable for [relational_product].
///
/// The read and write projections are arrays of indices that are read,
/// respectively written, by the corresponding sparse relation.
pub fn compute_meta(storage: &mut Storage, read_proj: &[u64], write_proj: &[u64]) -> Ldd
{
    // Compute length of meta.
    let length = cmp::max(
        match read_proj.iter().max()
        {
            Some(x) => *x+1,
            None => 0
        }
        , match write_proj.iter().max()
        {
            Some(x) => *x+1,
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
            meta.push(4);
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
/// relational_product(R, S, read_proj, write_proj) = { x[write_proj := y'] | project(x, read_proj) = x' and (x', y') in R and x in S }
///     where R is the relation and S the set.
///  
/// meta is a singleton vector where the value indicates the following for that index:
///     - 0 = not part the relation.
///     - 1 = only in read_proj
///     - 2 = only in write_proj
///     - 3 = in both read_proj and write_proj (read phase)
///     - 4 = in both read_proj and write_proj (write phase)
pub fn relational_product(storage: &mut Storage, set: &Ldd, rel: &Ldd, meta: &Ldd) -> Ldd
{
    assert_ne!(meta, storage.empty_set());

    if meta == storage.empty_vector() {
        // If meta is not defined then the rest is not in the relation (meta is always zero)
        set.clone()
    } else if set == storage.empty_set() || rel == storage.empty_set() {
        storage.empty_set().clone()
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
                        relational_product(storage, set, &rel_right, meta)
                    }
                }
            }
            2 => {
                // All values in set should be considered.
                let mut combined = storage.empty_set().clone(); 
                let mut current = set.clone();            
                loop {
                    let Data(_, set_down, set_right) = storage.get(&current);
                    combined = union(storage, &combined, &set_down);

                    if set_right == *storage.empty_set() {
                        break;
                    }
                    current = set_right;
                } 
                
                // Write the values present in the relation.
                let Data(rel_value, rel_down, rel_right) = storage.get(rel);

                let down_result = relational_product(storage, &combined, &rel_down, &meta_down);
                let right_result = relational_product(storage, set, &rel_right, meta);
                if down_result == *storage.empty_set()
                {
                    right_result
                } 
                else 
                {
                    storage.insert(rel_value, &down_result, &right_result)
                }
            }
            3 => {
                let Data(set_value, set_down, set_right) = storage.get(set);
                let Data(rel_value, rel_down, rel_right) = storage.get(rel);
                
                match set_value.cmp(&rel_value) {
                    Ordering::Less => {
                        relational_product(storage, &set_right, rel, meta)                        
                    }                    
                    Ordering::Equal => {
                        eprintln!("Matched {}", set_value);
                        let down_result = relational_product(storage, &set_down, &rel_down, &meta_down);
                        let right_result = relational_product(storage, &set_right, &rel_right, meta);
                        union(storage, &down_result, &right_result)
                    }
                    Ordering::Greater => {
                        relational_product(storage, &set, &rel_right, meta)
                    }
                }
            }
            4 => {                
                // Write the values present in the relation.
                let Data(rel_value, rel_down, rel_right) = storage.get(rel);

                let down_result = relational_product(storage, set, &rel_down, &meta_down);
                let right_result = relational_product(storage, set, &rel_right, meta);
                if down_result == *storage.empty_set()
                {
                    eprintln!("Down is empty");
                    right_result
                } 
                else 
                {
                    eprintln!("Wrote {}", rel_value);
                    storage.insert(rel_value, &down_result, &right_result)
                }
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
    use crate::fmt_node;
    use crate::test_utility::*;

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

    // Test the relational product function with read-only inputs.
    #[test]
    fn random_readonly_relational_product()
    {
        let mut storage = Storage::new();        
        let set = random_vector_set(32, 10, 10);

        let ldd =  from_iter(&mut storage, set.iter());

        let read_proj: Vec<u64> = vec![0,3,7,9];
        let meta = compute_meta(&mut storage, &read_proj, &[]);

        let proj_ldd = compute_proj(&mut storage, &read_proj);
        let relation = project(&mut storage, &ldd, &proj_ldd);

        let result = relational_product(&mut storage, &ldd, &relation, &meta);
        let read_project = project(&mut storage, &result, &proj_ldd);

        // relational_product(R, S, read_proj, []) = { x | project(x, read_proj) = x' and (x', <>) in R and x in S }
        assert_eq!(read_project, relation);
    }

    // Test the relational product function with write-only inputs.
    #[test]
    fn random_writeonly_relational_product()
    {
        let mut storage = Storage::new();        
        let set = random_vector_set(32, 10, 10);

        let ldd = from_iter(&mut storage, set.iter());

        let write_proj: Vec<u64> = vec![0,3,7,9];
        let meta = compute_meta(&mut storage, &[], &write_proj);

        let proj_ldd = compute_proj(&mut storage, &write_proj);
        let relation = project(&mut storage, &ldd, &proj_ldd);

        let result = relational_product(&mut storage, &ldd, &relation, &meta);
        let write_project = project(&mut storage, &result, &proj_ldd);

        // relational_product(R, S, [], write_proj) = { x[write_proj := y'] | (<>, y') in R and x in S }
        assert_eq!(write_project, relation);
    }

    #[test]
    fn random_relational_product()
    {
        let mut storage = Storage::new();      

        let set = random_vector_set(32, 10, 10);        
        let relation = random_vector_set(32, 4, 10);

        // Pick arbitrary read and write parameters in order.
        let read_proj = random_sorted_vector(2,9);
        let write_proj = random_sorted_vector(2,9);

        // The indices of the input vectors do not match the indices in the relation. The input vector is defined for all values, but the relation only
        // for relevant positions.
        let (read_rel_proj, write_rel_proj) = {
            let mut read_rel_proj: Vec<u64> = Vec::new();
            let mut write_rel_proj: Vec<u64> = Vec::new();

            let mut current = 0;
            for i in 0..10 
            {
                if read_proj.contains(&(i as u64)) {
                    read_rel_proj.push(current);
                    current += 1;
                }
                
                if  write_proj.contains(&(i as u64)) {
                    write_rel_proj.push(current);
                    current += 1;
                }
            }

            (read_rel_proj, write_rel_proj)
        };

        // Compute LDD result.
        let ldd = from_iter(&mut storage, set.iter());
        let rel = from_iter(&mut storage, relation.iter());

        let meta = compute_meta(&mut storage, &read_proj, &write_proj);
        let result = relational_product(&mut storage, &ldd, &rel, &meta);

        eprintln!("set = {}",  fmt_node(&storage, &ldd));
        eprintln!("relation = {}",  fmt_node(&storage, &rel));
        eprintln!("result = {}",  fmt_node(&storage, &result));
        eprintln!("========");

        eprintln!("meta = {}",  fmt_node(&storage, &meta));
        eprintln!("read {:?}, write {:?}, read_rel {:?} and write_rel {:?}", read_proj, write_proj, read_rel_proj, write_rel_proj);

        let expected = {
            let mut expected: HashSet<Vec<u64>> = HashSet::new();

            // Compute relational_product(R, S, read_proj, write_proj) = { x[write_proj := y'] | project(x, read_proj) = x' and (x', y') in R and x in S }
            for x in set.iter()
            {
                'next: for rel in relation.iter()
                {
                    let mut value: Vec<u64> = x.clone(); // The resulting vector.
                    let x_prime = project_vector(&rel, &read_rel_proj);
                    let y_prime = project_vector(&rel, &write_rel_proj);

                    // Ensure that project(x, read_proj) = x'
                    for (i, r) in read_proj.iter().enumerate()
                    {
                        if value[*r as usize] != x_prime[i] {
                            continue 'next;
                        }
                    }

                    // Compute x[write_proj := y']
                    for (i, w) in write_proj.iter().enumerate()
                    {
                        value[*w as usize] = y_prime[i];
                    }

                    // Print information about the value that we are testing.
                    eprintln!("value = {:?}, rel = {:?}", &value, &rel);
                    eprintln!("x_prime = {:?}, y_prime = {:?}", &x_prime, &y_prime);

                    assert!(element_of(&storage, &value, &result), "Result does not contain vector {:?}.", &value);
                    expected.insert(value);
                }
            }

            expected
        };

        // Check the other way around
        for res in iter(&storage, &result)
        {            
            assert!(expected.contains(&res), "Result unexpectedly contains vector {:?}.", res);
        }
    }

    // Test the project function with random inputs.
    #[test]
    fn random_project()
    {
        let mut storage = Storage::new();
        
        let set = random_vector_set(32, 10, 10);
        let proj: Vec<u64> = vec![0,3,7,9];

        let ldd = from_iter(&mut storage, set.iter());
        let proj_ldd = compute_proj(&mut storage, &proj);
        let result = project(&mut storage, &ldd, &proj_ldd);

        // Compute a naive projection on the vector set.
        let mut expected_result: HashSet<Vec<u64>> = HashSet::new();
        for element in &set
        {
            expected_result.insert(project_vector(element, &proj));
        }
        let expected = from_iter(&mut storage, expected_result.iter());
        assert_eq!(result, expected, "projected result does not match vector projection.");
    }
}
