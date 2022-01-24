use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::hash::{Hash, Hasher};

use crate::operations::height;

pub use self::ldd::Ldd;
use self::ldd::SharedStorage;

mod ldd;

/// This is the LDD node(value, down, right) with some additional meta data.
pub struct Node
{
    value: u64,
    down: usize,
    right: usize,

    reference_count: u64,
    marked: bool,
}

impl Node
{
    fn new(value: u64, down: usize, right: usize) -> Node
    {
        Node {value, down, right, reference_count: 0, marked: false}
    }
    
    /// Returns false if the node has been garbage collected.
    pub fn is_valid(&self) -> bool
    {
        !(self.down == 0 && self.right == 1) // These are values that can only be set during garbage collection.
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.down.hash(state);
        self.right.hash(state);
    }
}

/// This is the user-facing data of a Node.
pub struct Data(pub u64, pub Ldd, pub Ldd);

/// The storage that implements the maximal sharing behaviour. Meaning that
/// identical nodes (same value, down and right) have a unique index in the node
/// table. This means that Ldds n and m are identical iff their indices match.
pub struct Storage
{
    shared: Rc<RefCell<SharedStorage>>, // Every Ldd points to the underlying shared storage.
    index: HashMap<Node, usize>,
    free: Vec<usize>, // A list of free nodes.

    count_until_collection: u64, // Count down until the next garbage collection.
    enable_garbage_collection: bool, // Whether to enable automatic garbage collection based on heuristics.
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
        let shared = Rc::new(RefCell::new(SharedStorage::new()));
        let vector = vec![
                // Add two nodes representing 'false' and 'true' respectively; these cannot be created using insert.
                Node::new(0, 0, 0),
                Node::new(0, 0, 0),
            ];

        shared.borrow_mut().table = vector;

        Self { 
            index: HashMap::new(),
            shared: shared.clone(),
            free: vec![],
            count_until_collection: 10000,
            enable_garbage_collection: true,
            empty_set: Ldd::new(&shared, 0),
            empty_vector: Ldd::new(&shared, 1),
        }
    }

    /// Create a new LDD node(value, down, right)
    pub fn insert(&mut self, value: u64, down: &Ldd, right: &Ldd) -> Ldd
    {
        // These invariants ensure that the result is a valid LDD.
        debug_assert_ne!(down, self.empty_set(), "down node can never be the empty set.");
        debug_assert_ne!(right, self.empty_vector(), "right node can never be the empty vector."); 
        debug_assert!(down.index() < self.shared.borrow().table.len(), "down node not in table.");
        debug_assert!(right.index() < self.shared.borrow().table.len(), "right not not in table.");

        if right != self.empty_set()
        {
            debug_assert_eq!(height(self, down) + 1, height(self, right), "height should match the right node height.");
            debug_assert!(value < self.value(right), "value should be less than right node value.");
        }
        
        if self.count_until_collection == 0 
        {
            if self.enable_garbage_collection 
            {
                self.garbage_collect();
            }
            self.count_until_collection = self.shared.borrow().table.len() as u64;
        }

        let new_node = Node::new(value, down.index(), right.index());
        Ldd::new(&self.shared,
            *self.index.entry(new_node).or_insert_with(
            || 
            {
                let node = Node::new(value, down.index(), right.index());

                match self.free.pop()
                {
                    Some(index) => {
                        // Reuse existing position in table.
                        self.shared.borrow_mut().table[index] = node;
                        index
                    }
                    None => {
                        // No free positions so insert new.
                        self.count_until_collection -= 1;
                        self.shared.borrow_mut().table.push(node);
                        self.shared.borrow().table.len() - 1
                    }
                }
            })
        )
    }

    /// Cleans up all LDDs that are unreachable from the root LDDs.
    pub fn garbage_collect(&mut self)
    {
        println!("Collecting garbage...");

        // Mark all nodes that are (indirect) children of nodes with positive reference count.
        let num_of_elements = self.shared.borrow().table.len();
        for i in 0..num_of_elements
        {
            if self.shared.borrow().table[i].reference_count > 0
            {
                self.mark_node(i);
            }
        }
        
        // Collect all garbage.
        for (index, node) in self.shared.borrow_mut().table.iter_mut().enumerate()
        {
            if node.marked
            {
                node.marked = false
            }
            else
            {
                self.free.push(index);
                self.index.remove(node);

                // Insert garbage values so that the LDD is invalid (down node is empty set).
                node.value = 0;
                node.down = 0;
                node.right = 1;
            }
        }

        println!("Collected {} elements", self.free.len());
    /// Enables automatic garbage collection, which is enabled by default.
    pub fn enable_garbage_collection(&mut self, enabled: bool)
    {
        self.enable_garbage_collection = enabled;
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
    pub fn value(&self, ldd: &Ldd) -> u64
    {
        let node = &self.shared.borrow().table[ldd.index()];
        node.value
    }

    /// The down of an LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn down(&self, ldd: &Ldd) -> Ldd
    {
        self.verify_ldd(ldd);
        let node = &self.shared.borrow().table[ldd.index()];
        Ldd::new(&self.shared, node.down)
    }

    /// The right of an LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn right(&self, ldd: &Ldd) -> Ldd
    {
        self.verify_ldd(ldd);
        let node = &self.shared.borrow().table[ldd.index()];
        Ldd::new(&self.shared, node.right)
    }

    /// Returns a Data tuple for the given LDD node(value, down, right). Note, ldd cannot be 'true' or 'false.
    pub fn get(&self, ldd: &Ldd) -> Data
    {
        self.verify_ldd(ldd);     
        let data : (u64, usize, usize);
        {        
            let node = &self.shared.borrow().table[ldd.index()];
            data = (node.value, node.down, node.right);
        }

        Data(data.0, Ldd::new(&self.shared, data.1), Ldd::new(&self.shared, data.2))
    }

    // Asserts whether the given ldd is valid.
    fn verify_ldd(&self, ldd: &Ldd)
    {    
        debug_assert_ne!(ldd, self.empty_set(), "Cannot inspect empty set.");
        debug_assert_ne!(ldd, self.empty_vector(), "Cannot inspect empty vector.");  
        debug_assert!(self.shared.borrow().table[ldd.index()].is_valid(), "Node {} should not have been garbage collected", ldd.index());
    }
    
    /// Mark all LDDs reachable from the given root index.
    fn mark_node(&mut self, root: usize)
    {
        let mut stack: Vec<usize> = vec![root];
        let table = &mut self.shared.borrow_mut().table;
    
        while let Some(current) = stack.pop()
        {            
            let node = &mut table[current];
            if node.marked
            {
                continue
            }
            else
            {
                node.marked = true;
                if current != 0 && current != 1 {
                    stack.push(node.down);
                    stack.push(node.right);
                }
            }
        }
    }    

}

impl Drop for Storage
{
    fn drop(&mut self)
    {
        println!("There were {} reference count changes.", self.shared.borrow().reference_count_changes());
        println!("There were at most {} references to storage.", self.shared.borrow().max_references());
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_utility::*;
    use crate::operations::singleton;

    #[test]
    fn test_garbage_collection()
    {
        let mut storage = Storage::new();

        let _child: Ldd;
        {
            // Make sure that this set goes out of scope, but keep a reference to some child ldd.
            //let set = random_vector_set(1, 10);
            //let ldd = from_hashset(&mut storage, &set);
            let vector = random_vector(10, 10);
            let ldd = singleton(&mut storage, &vector);

            _child = storage.get(&ldd).1;
            storage.garbage_collect();
        }

        storage.garbage_collect();
    }
}