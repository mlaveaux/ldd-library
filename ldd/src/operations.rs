use crate::{Ldd, Storage, Data};

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
        let Data(a_value, a_down, a_right) = storage.get(a);
        let Data(b_value, b_down, b_right) = storage.get(b);

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

struct Iter<'a>
{
    storage: &'a Storage,
    current: Ldd,
}

fn iter(storage: &Storage, ldd: Ldd) -> Iter
{
    Iter {
        storage,
        current: ldd,
    }
}

impl Iterator for Iter<'_>
{
    type Item = Data;

    fn next(&mut self) -> Option<Self::Item>
    {             
        if self.current == self.storage.empty_set()
        {
            None
        }
        else
        {
            // Progress to the right LDD.
            let Data(value, down, right) = self.storage.get(self.current);       
            self.current = right;
            Some(Data(value, down, right))
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
        for Data(value, down, _) in iter(&storage, ldd)
        {            
            if value == vector[0] {
                return element_of(storage, &vector[1..], down)
            } else if value > vector[0] {
                return false
            }
        }

        false
    }    
}