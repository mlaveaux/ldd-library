
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::rc::Rc;

/// Every LDD points to its root node in the storage table.
pub struct Ldd
{
    index: usize, // Index in the node table.
    root: usize, // Index in the root set.
    storage: Rc<RefCell<ProtectionSet>>,
}

impl Ldd
{
    pub fn new(protect: &Rc<RefCell<ProtectionSet>>, index: usize) -> Ldd
    {
        let root = protect.borrow_mut().protect(index);
        let result = Ldd { storage: Rc::clone(protect), index, root };
        result
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
        Ldd::new(&self.storage, self.index)
    }
}

impl Drop for Ldd
{
    fn drop(&mut self)
    {
        self.storage.borrow_mut().unprotect(self.root);
    }
}

impl PartialEq for Ldd
{
    fn eq(&self, other: &Self) -> bool
    {
        debug_assert!(Rc::ptr_eq(&self.storage, &other.storage), "Both LDDs should refer to the same storage."); 
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

/// The protection set keeps track of LDD nodes that should not be garbage
/// collected, i.e., that are protected.
pub struct ProtectionSet
{    
    pub roots: Vec<usize>, // Every ldd has an index in this table that points to the node.
    free: Option<usize>,
    number_of_insertions: u64,
}

impl ProtectionSet
{
    pub fn new() -> Self
    {
        ProtectionSet { 
            roots: vec![],
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

    /// Protect the given ldd to prevent garbage collection.
    fn protect(&mut self, index: usize) -> usize
    {
        self.number_of_insertions += 1;

        match self.free {
            None => {
                // If free list is empty insert new entry into roots.
                self.roots.push(index);
                self.roots.len() - 1
            }
            Some(first) => {
                let next = self.roots[first];
                if first == next {
                    // The list is empty as its first element points to itself.
                    self.free = None;
                } else {
                    // Update free to be the next element in the list.
                    self.free = Some(next);
                }

                self.roots[first] = index;
                first
            }
        }
    }
    
    /// Remove protection from the given LDD.
    fn unprotect(&mut self, root: usize)
    {
        match self.free {
            None => {
                self.roots[root] = root;
            }
            Some(next) => {
                self.roots[root] = next;
            }
        };
        
        self.free = Some(root);
    }
}

impl Default for ProtectionSet {
    fn default() -> Self {
        Self::new()
    }
}    

impl Eq for Ldd {}