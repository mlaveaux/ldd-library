use std::collections::HashMap;
use crate::Ldd;

// This is the LDD node(value, down, right)
#[derive(PartialEq, Eq, Hash)]
struct Node
{
    value: u64,
    down: Ldd,
    right: Ldd
}

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
                    down: 0,
                    right: 0,
                },
                Node{
                    value: 0,
                    down: 0,
                    right: 0,
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
        assert!(down < self.table.len());
        assert!(right < self.table.len());

        if right != self.empty_set()
        {
            // Check that our height matches the right LDD.
            assert_eq!(self.height[down] + 1, self.height[right]);
            // Check that our value is less then the right value.
            assert!(value < self.value(right));
        }

        let new_node = Node {value, down, right};
        *self.index.entry(new_node).or_insert_with(
            || 
            {
                self.table.push(Node {value, down, right});
                self.height.push(self.height[down] + 1);
                self.table.len() - 1
            }
        )
    }

    // The 'false' LDD.
    pub fn empty_set(&self) -> Ldd
    {
        return 0
    }

    // The 'true' LDD.
    pub fn empty_vector(&self) -> Ldd
    {
        return 1
    }

    pub fn value(&self, ldd: Ldd) -> u64
    {
        self.table[ldd].value
    }

    pub fn get(&self, ldd: Ldd) -> (u64, Ldd, Ldd)
    {
        let node = &self.table[ldd];
        (node.value, node.down, node.right)
    }
}