
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp;

use crate::storage::Node;

/// Every LDD instance points to its root node in the storage.
pub struct Ldd
{
    index: usize,
    storage: Rc<RefCell<SharedStorage>>,
}

impl Ldd
{
    pub fn new(storage: &Rc<RefCell<SharedStorage>>, index: usize) -> Ldd
    {
        let result = Ldd { storage: Rc::clone(storage), index };
        storage.borrow_mut().protect( &result, Rc::strong_count(&storage));
        debug_assert!(storage.borrow().table[index].is_valid(), "Node {} should not have been garbage collected", index);
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
        self.storage.borrow_mut().unprotect(self);
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

/// Every LDD has shared access to the node table to adapt the reference counter.
pub struct SharedStorage
{    
    pub table: Vec<Node>,
    reference_count_changes: u64, // The number of times reference counters are changed.
    max_references: usize, // The maximum number of references to the shared storage.
}

impl SharedStorage
{
    pub fn new() -> Self
    {
        SharedStorage { 
            table: vec![], 
            reference_count_changes: 0, 
            max_references: 0 }
    }
    /// Returns total number of reference count changes.
    pub fn reference_count_changes(&self) -> u64 
    {
         self.reference_count_changes 
    }

    /// Returns maximum number of references to the shared storage (is equal to maximum number of active variables).
    pub fn max_references(&self) -> usize 
    {
        self.max_references 
    }

    /// Protect the given ldd to prevent garbage collection.
    fn protect(&mut self, ldd: &Ldd, count: usize)
    {
        self.reference_count_changes += 1;
        self.max_references = cmp::max(self.max_references, count);
        self.table[ldd.index].reference_count += 1;
    }
    
    /// Remove protection from the given LDD.
    fn unprotect(&mut self, ldd: &Ldd)
    {
        self.reference_count_changes += 1;
        self.table[ldd.index].reference_count -= 1;
    }
}

impl Eq for Ldd {}