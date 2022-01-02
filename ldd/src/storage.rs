use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};

/// Every LDD instance points to its root node in the storage.
pub struct Ldd
{
    index: usize,
    storage: Rc<RefCell<SharedStorage>>,
}

impl Ldd
{
    fn new(storage: &Rc<RefCell<SharedStorage>>, index: usize) -> Ldd
    {
        let result = Ldd { storage: Rc::clone(storage), index };
        storage.borrow_mut().protect(&result);
        result
    }

    pub fn index(self: &Self) -> usize
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
        assert!(Rc::ptr_eq(&self.storage, &other.storage)); // Both LDDs should refer to the same storage.
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

impl Eq for Ldd {}

/// This is the LDD node(value, down, right) with some additional meta data.
struct Node
{
    value: u64,
    down: usize,
    right: usize,

    reference_count: u64,
    marked: bool,
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

impl Node
{
    fn new(value: u64, down: usize, right: usize) -> Node
    {
        Node {value, down, right, reference_count: 0, marked: false}
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
    empty_set: Ldd,
    empty_vector: Ldd,
    
    height: Vec<u64>, // Used for debugging to ensure that the created LDDs are valid.
    //reference_count_changes: u64, // The number of times reference counters are changed.
}

// Gives every node shared access to the underlying table.
pub struct SharedStorage
{    
    table: Vec<Node>,
}

const EMPTY_SET: usize = 0;
const EMPTY_VECTOR: usize = 1;

impl Storage
{
    pub fn new() -> Self
    {
        let shared = Rc::new(RefCell::new(SharedStorage { table: vec![] }));
        let vector = vec![
                // Add two nodes representing 'false' and 'true' respectively; these cannot be created using insert.
                Node::new(0, 0, 0),
                Node::new(0, 0, 0),
            ];

        shared.borrow_mut().table = vector;

        let library = Self { 
            index: HashMap::new(),
            shared: shared.clone(),
            // Only used for debugging purposes. height(false) = 0 and height(true) = 0, note that height(false) is irrelevant
            free: vec![],
            height: vec![0, 0],
            empty_set: Ldd::new(&shared, EMPTY_SET),
            empty_vector: Ldd::new(&shared, EMPTY_VECTOR),
        };
               
        library
    }

    // Create a new node(value, down, right)
    pub fn insert(&mut self, value: u64, down: &Ldd, right: &Ldd) -> Ldd
    {
        // Check the validity of the down and right nodes.
        assert_ne!(down, self.empty_set());
        assert_ne!(right, self.empty_vector()); 
        assert!(down.index < self.shared.borrow().table.len());
        assert!(right.index < self.shared.borrow().table.len());

        if right != self.empty_set()
        {
            // Check that our height matches the right LDD.
            assert_eq!(self.height[down.index] + 1, self.height[right.index]);
            // Check that our value is less then the right value.
            assert!(value < self.value(&right));
        }

        let new_node = Node::new(value, down.index, right.index);
        Ldd::new(&self.shared,
            *self.index.entry(new_node).or_insert_with(
            || 
            {
                let node = Node::new(value, down.index, right.index);

                match self.free.pop()
                {
                    Some(index) => {
                        // Reuse existing position in table.
                        self.shared.borrow_mut().table[index] = node;
                        self.height[index] = self.height[down.index] + 1;
                        index
                    }
                    None => {
                        // No free positions so insert new.
                        self.shared.borrow_mut().table.push(node);
                        self.height.push(self.height[down.index] + 1);
                        self.shared.borrow().table.len() - 1
                    }
                }
            }
            )
        )
    }

    fn mark_node(&mut self, root: usize)
    {
        let table = &mut self.shared.borrow_mut().table;
        let mut stack: Vec<usize> = vec![root];

        loop
        {
            let current = match stack.pop() {
                Some(x) => x,
                None => break,
            };
            
            let node = &mut table[current];
            if node.marked
            {
                continue
            }
            else
            {
                node.marked = true;
                stack.push(node.down);
                stack.push(node.right);
            }
        }
    }

    pub fn garbage_collect(&mut self)
    {
        let mut roots: Vec<usize> = vec![];

        // Mark all nodes that are (indirect) children of nodes with positive reference count.
        for (index, node) in &mut self.shared.borrow_mut().table.iter_mut().enumerate()
        {
            if node.reference_count > 0
            {
                roots.push(index);
            }
        }

        for root in roots.iter()
        {
            self.mark_node(*root);
        }
        
        // Collect all garbage.
        for (index, node) in &mut self.shared.borrow_mut().table.iter_mut().enumerate()
        {
            if node.marked
            {
                node.marked = false
            }
            else
            {
                self.free.push(index);

                // Insert garbage values so an error is produced when it is used.
                node.value = 0;
                node.down = 0;
                node.right = 0;
            }
        }        

        // Check that freed nodes do not occur in any other LDD.
        for garbage in self.free.iter()
        {
            for node in self.shared.borrow().table.iter()
            {
                assert_ne!(node.down, *garbage);
                assert_ne!(node.right, *garbage);
            }
        }
    }

    // The 'false' LDD.
    pub fn empty_set(&self) -> &Ldd
    {
        &self.empty_set
    }

    // The 'true' LDD.
    pub fn empty_vector(&self) -> &Ldd
    {
        &self.empty_vector
    }

    pub fn value(&self, ldd: &Ldd) -> u64
    {
        self.shared.borrow().table[ldd.index].value
    }

    pub fn get(&self, ldd: &Ldd) -> Data
    {
        let data : (u64, usize, usize);
        {        
            let node = &self.shared.borrow().table[ldd.index];
            // Ensure that this node is valid as garbage collection will insert garbage values.
            assert_ne!(node.down, EMPTY_SET);
            assert_ne!(node.right, EMPTY_VECTOR); 

            data = (node.value, node.down, node.right);
        }

        Data(data.0, Ldd::new(&self.shared, data.1), Ldd::new(&self.shared, data.2))
    }
}

impl SharedStorage
{
    // Protect the given ldd to prevent garbage collection.
    fn protect(&mut self, ldd: &Ldd)
    {
        self.table[ldd.index].reference_count += 1
    }
    
    // Remove protection from the given LDD.
    fn unprotect(&mut self, ldd: &Ldd)
    {
        self.table[ldd.index].reference_count -= 1
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::common::*;
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
            let vector = random_vector(10);
            let ldd = singleton(&mut storage, &vector);

            _child = storage.get(&ldd).1;
            storage.garbage_collect();
        }

        storage.garbage_collect();
    }
}