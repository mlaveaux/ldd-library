
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::rc::Rc;

/// Every LDD points to its root node in the [Storage] instance for maximum sharing.
pub struct Ldd
{
    index: usize, // Index in the node table.
    root: usize, // Index in the root set.
    protection_set: Rc<RefCell<ProtectionSet>>,
}

impl Ldd
{
    pub fn new(protection_set: &Rc<RefCell<ProtectionSet>>, index: usize) -> Ldd
    {
        let root = protection_set.borrow_mut().protect(index);
        Ldd { protection_set: Rc::clone(protection_set), index, root }
    }

    pub fn index(&self) -> usize
    {
        self.index
    }
}


impl Clone for Ldd
{
    fn clone(&self) -> Self
    {
        Ldd::new(&self.protection_set, self.index)
    }
}

impl Drop for Ldd
{
    fn drop(&mut self)
    {
        self.protection_set.borrow_mut().unprotect(self.root);
    }
}

impl PartialEq for Ldd
{
    fn eq(&self, other: &Self) -> bool
    {
        debug_assert!(Rc::ptr_eq(&self.protection_set, &other.protection_set), "Both LDDs should refer to the same storage."); 
        self.index == other.index
    }
}

impl Debug for Ldd
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "index: {}", self.index)
    }
}

impl Hash for Ldd
{    
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index().hash(state);
    }
}

impl Eq for Ldd {}

/// The protection set keeps track of LDD nodes that should not be garbage
/// collected, i.e., that are protected.
#[derive(Default)]
pub struct ProtectionSet
{    
    roots: Vec<(usize, bool)>, // Every ldd has an index in this table that points to the node.
    free: Option<usize>,
    number_of_insertions: u64,
}

impl ProtectionSet
{
    pub fn new() -> Self
    {
        ProtectionSet { 
            roots: Vec::new(),
            free: None,
            number_of_insertions: 0,
        }
    }

    /// Returns The number of insertions into the protection set.
    pub fn number_of_insertions(&self) -> u64 
    {
        self.number_of_insertions 
    }

    /// Returns maximum number of active ldd instances.
    pub fn maximum_size(&self) -> usize 
    {
        self.roots.capacity() 
    }

    /// Returns an iterator over all root indices in the protection set.
    pub fn iter(&self) -> ProtSetIter
    {
        ProtSetIter {
            current: 0,
            protection_set: self,
        }
    }

    /// Protect the given node to prevent garbage collection.
    fn protect(&mut self, index: usize) -> usize
    {
        self.number_of_insertions += 1;

        match self.free {
            None => {
                // If free list is empty insert new entry into roots.
                self.roots.push((index, true));
                self.roots.len() - 1
            }
            Some(first) => {
                let next = self.roots[first];
                if first == next.0 {
                    // The list is empty as its first element points to itself.
                    self.free = None;
                } else {
                    // Update free to be the next element in the list.
                    self.free = Some(next.0);
                }

                self.roots[first] = (index, true);
                first
            }
        }
    }
    
    /// Remove protection from the given LDD, note that root is here the index returned by [protect].
    fn unprotect(&mut self, root: usize)
    {
        match self.free {
            None => {
                self.roots[root] = (root, false);
            }
            Some(next) => {
                self.roots[root] = (next, false);
            }
        };
        
        self.free = Some(root);
    }
}

pub struct ProtSetIter<'a>
{
    current: usize,
    protection_set: &'a ProtectionSet,
}

impl Iterator for ProtSetIter<'_>
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item>
    {
        // Find the next valid entry, return it when found or None when end of roots is reached.
        while self.current < self.protection_set.roots.len()
        {
            let (root, valid) = self.protection_set.roots[self.current];
            self.current += 1;
            if valid {
                return Some(root);
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_utility::*;

    #[test]
    fn test_protection_set()
    {
        let mut protection_set = ProtectionSet::new();

        // Protect a number of LDDs and record their indices.
        let root_variables = random_vector(1000, 5000);
        let mut indices: Vec<usize> = Vec::new();

        for variable in root_variables
        {
            indices.push(protection_set.protect(variable as usize));
        }

        // Unprotect a number of LDDs.
        for index in 0..250
        {
            protection_set.unprotect(indices[index]);
            indices.remove(index);
        }
        
        for index in indices
        {
            let (_, valid) = protection_set.roots[index];
            assert!(valid, "All indices that are not unprotected should occur in the protection set");
        }

        for root in protection_set.iter()
        {
            assert!(root <= 5000, "Root must be valid");
        }
    }
}
