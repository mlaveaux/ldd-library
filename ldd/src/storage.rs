use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};

// Every LDD points to its root node by means of an index and it has the shared storage for the protection.
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
}

impl Clone for Ldd
{
    fn clone(&self) -> Self
    {
        self.storage.borrow_mut().protect(self);
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

// This is the LDD node(value, down, right) with a reference counter.
struct Node
{
    value: u64,
    down: usize,
    right: usize,
    reference_count: u64,
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
        Node {value, down, right, reference_count: 0}
    }
}

// This is only the user-facing data of a Node.
pub struct Data(pub u64, pub Ldd, pub Ldd);

// The storage that implements the maximal sharing behaviour. Meaning that
// identical nodes (same value, down and right) have a unique index in the node
// table. Therefore Ldds n and m are identical iff their indices match.
pub struct Storage
{
    shared: Rc<RefCell<SharedStorage>>, // Every Ldd points to the underlying shared storage.
    index: HashMap<Node, usize>,
    height: Vec<u64>,
    empty_set: Ldd,
    empty_vector: Ldd,
}

// Gives every node shared access to the underlying table.
pub struct SharedStorage
{    
    table: Vec<Node>,
}

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
            height: vec![0, 0],
            empty_set: Ldd::new(&shared, 0),
            empty_vector: Ldd::new(&shared, 1),
        };
               
        library
    }

    // Create a new node(value, down, right)
    pub fn insert(&mut self, value: u64, down: Ldd, right: Ldd) -> Ldd
    {
        // Check the validity of the down and right nodes.
        assert_ne!(down, *self.empty_set());
        assert_ne!(right, *self.empty_vector()); 
        assert!(down.index < self.shared.borrow().table.len());
        assert!(right.index < self.shared.borrow().table.len());

        if right != *self.empty_set()
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
                self.shared.borrow_mut().table.push(node);
                self.height.push(self.height[down.index] + 1);
                self.shared.borrow().table.len() - 1
            }
            )
        )
    }

    pub fn garbage_collect(&mut self)
    {
        /*for node in enumerate(&self.shared.borrow_mut().table[..])
        {
            if node.reference_count == 0
            {
                println!("Node {} is garbage", )
            }
        }*/
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
    
    fn unprotect(&mut self, ldd: &Ldd)
    {
        self.table[ldd.index].reference_count -= 1
    }
}
