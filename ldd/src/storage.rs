use std::collections::HashMap;

// Every LDD points to its root node by means of an index.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Ldd
{
    index: usize,
}

impl Ldd
{
    fn new(index: usize) -> Ldd
    {
        Ldd { index }
    }
}

impl Clone for Ldd
{
    fn clone(&self) -> Self
    {
        Ldd { index: self.index, }
    }
}

// This is the LDD node(value, down, right)
#[derive(PartialEq, Eq, Hash)]
struct Node
{
    value: u64,
    down: Ldd,
    right: Ldd
}

// This is only the user-facing data of a Node.
pub struct Data(pub u64, pub Ldd, pub Ldd);

// The storage that implements the maximal sharing behaviour. Meaning that identical nodes (same value, down and right) have a unique index in the node table. Therefore Ldds n and m are identical iff their indices match.
pub struct Storage
{
    index: HashMap<Node, usize>,
    table: Vec<Node>,
    height: Vec<u64>,
}

impl Storage
{
    pub fn new() -> Self
    {
        let mut library = Self { 
            index: HashMap::new(),
            table: vec![
                 // Add two nodes representing 'false' and 'true' respectively; these cannot be created using insert.
                Node{
                    value: 0,
                    down: Ldd::new(0),
                    right: Ldd::new(0),
                },
                Node{
                    value: 0,
                    down: Ldd::new(0),
                    right: Ldd::new(0),
                }
            ],
            height: Vec::new(),
        };
       
        // Only used for debugging purposes. height(false) = 0 and height(true) = 0, note that height(false) is irrelevant
        library.height.push(0);
        library.height.push(0);
        
        library
    }

    // Create a new node(value, down, right)
    pub fn insert(&mut self, value: u64, down: Ldd, right: Ldd) -> Ldd
    {
        // Check the validity of the down and right nodes.
        assert_ne!(down, self.empty_set());
        assert_ne!(right, self.empty_vector());
        assert!(down.index < self.table.len());
        assert!(right.index < self.table.len());

        if right != self.empty_set()
        {
            // Check that our height matches the right LDD.
            assert_eq!(self.height[down.index] + 1, self.height[right.index]);
            // Check that our value is less then the right value.
            assert!(value < self.value(&right));
        }

        let new_node = Node {value, down: down.clone(), right: right.clone()};
        Ldd
        {
            index: *self.index.entry(new_node).or_insert_with(
            || 
            {
                self.table.push(Node 
                    {
                        value, 
                        down: Ldd::new(down.index), 
                        right: Ldd::new(right.index),
                    });
                self.height.push(self.height[down.index] + 1);
                self.table.len() - 1
            }
            )
        }
    }

    // The 'false' LDD.
    pub fn empty_set(&self) -> Ldd
    {
        Ldd::new(0)
    }

    // The 'true' LDD.
    pub fn empty_vector(&self) -> Ldd
    {
        Ldd::new(1)
    }

    pub fn value(&self, ldd: &Ldd) -> u64
    {
        self.table[ldd.index].value
    }

    pub fn get(&self, ldd: &Ldd) -> Data
    {
        let node = &self.table[ldd.index];
        Data(node.value, node.down.clone(), node.right.clone())
    }
}