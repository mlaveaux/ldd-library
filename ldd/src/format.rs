use crate::{Ldd, Storage, iterators::*};

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
    for vector in iter(storage, ldd) 
    {
        // Here, we have found another vector in the LDD.
        write!(f, "<")?;
        for val in vector
        {
            write!(f, "{} ", val)?;
        }
        write!(f, ">\n")?;
    }

    Ok(())
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
