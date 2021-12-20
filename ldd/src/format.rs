
use crate::Ldd;
use crate::Storage;

use std::fmt;

// Return a formatter for the given Ldd.
pub fn fmt_node(storage: &Storage, ldd: Ldd) -> Display
{
    Display {
        storage,
        ldd,
    }
}

// Print the lists represented by the given LddNode.
pub struct Display<'a>
{
    storage: &'a Storage,
    ldd: Ldd,
}

fn print(storage: &Storage, cache: &mut Vec<u64>, ldd: Ldd, f: &mut fmt::Formatter<'_>) -> fmt::Result
{
    if ldd == storage.empty_set() {
        Ok(())
    } 
    else if ldd == storage.empty_vector() 
    {
        // Here, we have found another vector in the LDD.
        write!(f, "<")?;
        for val in cache
        {
            write!(f, "{} ", val)?;
        }
        write!(f, ">\n")
    }
    else
    {
        // Loop over all nodes on this level
        let mut current = ldd;

        loop
        {
            let (value, down, right) = storage.get(current);

            cache.push(value);
            print(storage, cache, down, f)?;
            cache.pop();

            if right == storage.empty_set()
            {
                break
            }
            else
            {
                current = right;
            }
        }
        Ok(())        
    }
}

impl fmt::Display for Display<'_>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let mut cache: Vec<u64> = Vec::new();

        write!(f, "{{ \n")?;
        print(self.storage, &mut cache, self.ldd, f)?;
        write!(f, "}}")
    }
}
