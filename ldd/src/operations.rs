use crate::{Ldd, Storage};

use std::cmp::Ordering;

// Returns an LDD containing only the given vector, i.e., { vector }
pub fn singleton(storage: &mut Storage, vector: &[u64]) -> Ldd
{
    let mut root = storage.empty_vector();
    for val in vector.iter().rev()
    {
        root = storage.insert(*val, root, storage.empty_set());
    }

    root
}

// Returns the union of the given LDDs.
pub fn union(storage: &mut Storage, a: Ldd, b: Ldd) -> Ldd
{
    if a == b {
        a
    } else if a == storage.empty_set() {
        b
    } else if b == storage.empty_set() {
        a
    } else {
        let (a_value, a_down, a_right) = storage.get(a);
        let (b_value, b_down, b_right) = storage.get(b);

        match a_value.cmp(&b_value) {
            Ordering::Less => {
                let result = union(storage, a_right, b);
                storage.insert(a_value, a_down, result)
            },
            Ordering::Equal => {
                let down_result = union(storage, a_down, b_down);
                let right_result = union(storage, a_right, b_right);
                storage.insert(a_value, down_result, right_result)
            },
            Ordering::Greater => {
                let result = union(storage, a, b_right);
                storage.insert(b_value, b_down, result)
            }
        }
    }
}

// Returns true iff the given vector is included in the LDD.
pub fn element_of(storage: &Storage, vector: &[u64], ldd: Ldd) -> bool
{
    if vector.len() == 0
    {
        ldd == storage.empty_vector()
    }
    else if ldd == storage.empty_vector()
    {
        false
    }
    else
    {
        // Loop over all nodes on this level
        let mut current = ldd;
        loop
        {        
            let (value, down, right) = storage.get(current);
            if value == vector[0]
            {
                break element_of(storage, &vector[1..], down)
            }
    
            if right == storage.empty_set()
            {
                break false
            }
            else
            {
                current = right;
            }
        }
    }    
}