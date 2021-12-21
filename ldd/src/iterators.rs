use crate::{Ldd, Storage, Data};

// Returns an iterator over all right siblings of the given LDD.
pub fn iter_right(storage: &Storage, ldd: Ldd) -> IterRight
{
    IterRight {
        storage,
        current: ldd,
    }
}

// Returns an iterator over all vectors contained in the given LDD.
pub fn iter(storage: &Storage, ldd: Ldd) -> Iter
{       
    if ldd == storage.empty_set() {        
        Iter {
            storage,
            vector: Vec::new(),
            stack: Vec::new(),
        }
    } else {
        Iter {
            storage,
            vector: Vec::new(),
            stack: vec![ldd],
        }
    }
}

pub struct IterRight<'a>
{
    storage: &'a Storage,
    current: Ldd,
}

impl Iterator for IterRight<'_>
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

pub struct Iter<'a>
{
    storage: &'a Storage,
    vector: Vec<u64>, // Stores the values of the returned vector.
    stack: Vec<Ldd>, // Stores the stack for the depth-first search (only non 'true' or 'false' nodes)
}

impl Iterator for Iter<'_>
{
    type Item = Vec<u64>;

    fn next(&mut self) -> Option<Self::Item>
    { 
        // Find the next vector by going down the chain.
        let vector: Vec<u64>;     
        loop
        {
            let current = match self.stack.last() {
                Some(x) => x,
                None => return None,
            };

            let Data(value, down, _) = self.storage.get(*current);
            self.vector.push(value);
            if down == self.storage.empty_vector()
            {
                vector = self.vector.clone();
                break; // Stop iteration.
            }
            else 
            {
                self.stack.push(down);
            }
        }

        // Go up the chain to find the next right sibling that is not 'false'.    
        loop
        {
            let current = match self.stack.pop() {
                Some(x) => x,
                None => break,
            };

            self.vector.pop();
            let Data(_, _, right) = self.storage.get(current);

            if right != self.storage.empty_set()
            {
                self.stack.push(right); // This is the first right sibling.
                break;
            }           
        }

        Some(vector)
    }
}