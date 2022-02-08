use std::cell::RefCell;
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHashMap;

use crate::operations::height;

mod ldd;
mod cache;

pub use self::cache::*;
pub use self::ldd::{Ldd, LddRef};
use self::ldd::{ProtectionSet};

pub type Value = u32;

/// This is the LDD node(value, down, right) with some additional meta data.
pub struct Node
{
    value: Value,
    down: usize,
    right: usize, // If !filled then right is the next freelist element.

    marked: bool,
    filled: bool, // Indicates whether this position in the table represents a valid node.
}

static_assertions::assert_eq_size!(Node, (usize, usize, usize));

impl Node
{
    fn new(value: Value, down: usize, right: usize) -> Node
    {
        Node {value, down, right, marked: false, filled: true}
    }
    
    /// Returns false if the node has been garbage collected.
    pub fn is_valid(&self) -> bool
    {
        self.filled        
    }
}

impl PartialEq for Node
{
    fn eq(&self, other: &Self) -> bool
    {
        self.value == other.value && self.down == other.down && self.right == other.right
    }
}

impl Eq for Node {}

impl Hash for Node
{    
    fn hash<H: Hasher>(&self, state: &mut H) 
    {
        self.value.hash(state);
        self.down.hash(state);
        self.right.hash(state);
    }
}

/// This is the user facing data of a [Node].
pub struct Data(pub Value, pub Ldd, pub Ldd);

/// This is the user facing data of a [Node] as references.
pub struct DataRef<'a>(pub Value, pub LddRef<'a>, pub LddRef<'a>);

/// The storage that implements the maximal sharing behaviour. Meaning that
/// identical nodes (same value, down and right) have a unique index in the node
/// table. Therefore guaranteeing that Ldds n and m are identical iff their
/// indices in the node table match.
pub struct Storage
{
    protection_set: Rc<RefCell<ProtectionSet>>, // Every Ldd points to the underlying protection set.
    table: Vec<Node>,
    index: FxHashMap<Node, usize>,
    cache: OperationCache,

    free: Option<usize>, // A list of free nodes.

    count_until_collection: u64, // Count down until the next garbage collection.
    enable_garbage_collection: bool, // Whether to enable automatic garbage collection based on heuristics.
    enable_performance_metrics: bool,
    empty_set: Ldd,
    empty_vector: Ldd,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}   

impl Storage
{
    pub fn new() -> Self
    {
        let shared = Rc::new(RefCell::new(ProtectionSet::new()));

        Self { 
            index: FxHashMap::default(),
            protection_set: shared.clone(),
            table:  vec![
                // Add two nodes representing 'false' and 'true' respectively; these cannot be created using insert.
                Node::new(0, 0, 0),
                Node::new(0, 0, 0),
                ],
            cache: OperationCache::new(Rc::clone(&shared)),

            count_until_collection: 10000,
            free: None,
            enable_garbage_collection: true,
            enable_performance_metrics: false,
            empty_set: Ldd::new(&shared, 0),
            empty_vector: Ldd::new(&shared, 1),
        }
    }

    /// Provides access to the underlying operation cache.
    pub fn operation_cache(&mut self) -> &mut OperationCache
    {
        &mut self.cache
    }

    /// Create a new LDD node(value, down, right)
    pub fn insert(&mut self, value: Value, down: LddRef, right: LddRef) -> Ldd
    {
        // These invariants ensure that the result is a valid LDD.
        debug_assert_ne!(down, *self.empty_set(), "down node can never be the empty set.");
        debug_assert_ne!(right, *self.empty_vector(), "right node can never be the empty vector."); 
        debug_assert!(down.index() < self.table.len(), "down node not in table.");
        debug_assert!(right.index() < self.table.len(), "right not not in table.");

        if right != *self.empty_set()
        {
            debug_assert_eq!(height(self, down.borrow()) + 1, height(self, right.borrow()), 
                "height of node {} should match the right node {} height.", down.index(), right.index());
            debug_assert!(value < self.value(right.borrow()), "value should be less than right node value.");
        }
        
        if self.count_until_collection == 0 
        {
            if self.enable_garbage_collection 
            {
                self.garbage_collect();
            }
            self.count_until_collection = self.table.len() as u64;
        }
        
        let new_node = Node::new(value, down.index(), right.index());
        let index = *self.index.entry(new_node).or_insert_with(
            || 
            {
                let node = Node::new(value, down.index(), right.index());

                match self.free {
                    Some(first) =>
                    {
                        let next = self.table[first].right;
                        if first == next {
                            // The list is now empty as its first element points to itself.
                            self.free = None;
                        } else {
                            // Update free to be the next element in the list.
                            self.free = Some(next);
                        }
        
                        self.table[first] = node;
                        first
                    }
                    None =>
                    {
                        // No free positions so insert new.
                        self.count_until_collection -= 1;
                        self.table.push(node);
                        self.table.len() - 1
                    }
                }
            });
        
        Ldd::new(&self.protection_set, index)
    }

    /// Upgrade an [LddRef] to a protected [Ldd] instance.
    pub fn protect(&mut self, ldd: LddRef) -> Ldd
    {
        Ldd::new(&self.protection_set, ldd.index())
    }

    /// Cleans up all LDDs that are unreachable from the root LDDs.
    pub fn garbage_collect(&mut self)
    {
        // Clear the cache since it contains unprotected LDDs, and keep track of size before clearing.
        let size_of_cache = self.cache.len();
        self.cache.clear();

        // Mark all nodes that are (indirect) children of nodes with positive reference count.
        let mut stack: Vec<usize> = Vec::new();
        for root in self.protection_set.borrow().iter()
        {
            mark_node(&mut self.table, &mut stack, root);
        }
        
        // Collect all garbage.
        let mut number_of_collections: usize = 0;
        for (index, node) in self.table.iter_mut().enumerate()
        {
            if node.marked
            {
                debug_assert!(node.is_valid(), "Should never mark a node that is not valid.");
                node.marked = false
            }
            else
            {
                self.index.remove(node); // First rmove the node as mutating it changes the hash.
                                
                match self.free {
                    Some(next) => {
                        node.right = next;
                    }
                    None => {
                        node.right = index;
                    }
                };
                self.free = Some(index);
                node.filled = false;

                number_of_collections += 1;
            }
        }

        // Check whether the direct children of a valid node are valid (this implies that the whole tree is valid if the root is valid).
        for node in self.table.iter()
        {
            if node.is_valid()
            {
                debug_assert!(self.table[node.down].is_valid(), "The down node of a valid node must be valid.");
                debug_assert!(self.table[node.right].is_valid(), "The right node of a valid node must be valid.");
            }
        }

        if self.enable_performance_metrics {
            println!("Collected {number_of_collections} elements and {} elements remaining", self.table.len());
            println!("Operation cache contains {size_of_cache} elements");
        }
    }
    
    /// Enables automatic garbage collection, which is enabled by default.
    pub fn enable_garbage_collection(&mut self, enabled: bool)
    {
        self.enable_garbage_collection = enabled;
    }

    pub fn enable_performance_metrics(&mut self, enabled: bool)
    {
        self.enable_performance_metrics = enabled;
    }

    /// The 'false' LDD.
    pub fn empty_set(&self) -> &Ldd
    {
        &self.empty_set
    }

    /// The 'true' LDD.
    pub fn empty_vector(&self) -> &Ldd
    {
        &self.empty_vector
    }

    /// The value of an LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn value(&self, ldd: LddRef) -> Value
    {
        self.verify_ldd(ldd.borrow());
        let node = &self.table[ldd.index()];
        node.value
    }

    /// The down of an LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn down(&self, ldd: LddRef) -> Ldd
    {
        self.verify_ldd(ldd.borrow());
        let node = &self.table[ldd.index()];
        Ldd::new(&self.protection_set, node.down)
    }

    /// The right of an LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn right(&self, ldd: LddRef) -> Ldd
    {
        self.verify_ldd(ldd.borrow());
        let node = &self.table[ldd.index()];
        Ldd::new(&self.protection_set, node.right)
    }

    /// Returns a Data tuple for the given LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn get(&self, ldd: LddRef) -> Data
    {
        self.verify_ldd(ldd.borrow());     
        let node = &self.table[ldd.index()];
        Data(node.value, Ldd::new(&self.protection_set, node.down), Ldd::new(&self.protection_set, node.right))
    }

    /// Returns a DataRef tuple for the given LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn get_ref<'a>(&self, ldd: LddRef<'a>) -> DataRef<'a>
    {
        self.verify_ldd(ldd.borrow());     
        let node = &self.table[ldd.index()];
        DataRef(node.value, LddRef::new(node.down), LddRef::new(node.right))
    }

    // Asserts whether the given ldd is valid.
    fn verify_ldd(&self, ldd: LddRef)
    {    
        debug_assert_ne!(&ldd, self.empty_set(), "Cannot inspect empty set.");
        debug_assert_ne!(&ldd, self.empty_vector(), "Cannot inspect empty vector.");  
        debug_assert!(self.table[ldd.index()].is_valid(), "Node {} should not have been garbage collected", ldd.index());
    }
}

impl Drop for Storage
{
    fn drop(&mut self)
    {
        if self.enable_performance_metrics {
            println!("There were {} insertions into the protection set.", self.protection_set.borrow().number_of_insertions());
            println!("There were at most {} root variables.", self.protection_set.borrow().maximum_size());
            println!("There were at most {} nodes.", self.table.capacity());
        }
    }
}

/// Mark all LDDs reachable from the given root index.
/// 
/// Reuses the stack for the depth-first exploration.
fn mark_node(table: &mut Vec<Node>, stack: &mut Vec<usize>, root: usize)
{
    stack.push(root);
    while let Some(current) = stack.pop()
    {            
        let node = &mut table[current];
        debug_assert!(node.is_valid(), "Should never mark a node that is not valid.");
        if node.marked
        {
            continue
        }
        else
        {
            node.marked = true;
            if current != 0 && current != 1
            {
                stack.push(node.down);
                stack.push(node.right);
            }
        }
    }
    
    debug_assert!(stack.is_empty(), "When marking finishes the stack should be empty.");
} 

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_utility::*;
    use crate::operations::singleton;

    #[test]
    fn test_garbage_collection_small()
    {
        let mut storage = Storage::new();

        let _child: Ldd;
        {
            // Make sure that this set goes out of scope, but keep a reference to some child ldd.
            let vector = random_vector(10, 10);
            let ldd = singleton(&mut storage, &vector);

            _child = storage.get(ldd.borrow()).1;
            storage.garbage_collect();
        }

        storage.garbage_collect();
    }
    
    #[test]
    fn test_garbage_collection()
    {
        let mut storage = Storage::new();

        let _child: Ldd;
        {
            // Make sure that this set goes out of scope, but keep a reference to some child ldd.
            let vector = random_vector_set(2000, 10, 2);
            let ldd = from_iter(&mut storage, vector.iter());

            _child = storage.get(storage.get(ldd.borrow()).1.borrow()).1;
            storage.garbage_collect();
        }

        storage.garbage_collect();
    }
}
