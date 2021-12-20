use std::collections::HashMap;
use std::fmt;
use std::cmp::Ordering;

// List Decision Diagrams, abbreviated LDD, are data structures that represent sets of fixed length vectors. 
// An LDD represents a set as follows. Given an LDD n then [[n]] is defined as:
// [[false]] = emptyset
// [[true]] = { <> } (the singleton set containing the empty vector)
// [[node(value, down, right)]] = { value x | x in [[down]] } union [[right]] (where value x is the concatenation of vectors)

// Every LDD points to its root node by means of an index.
pub type Ldd = usize;

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
    fn insert(&mut self, value: u64, down: Ldd, right: Ldd) -> Ldd
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

    fn value(&self, ldd: Ldd) -> u64
    {
        self.table[ldd].value
    }

    fn get(&self, ldd: Ldd) -> (u64, Ldd, Ldd)
    {
        let node = &self.table[ldd];
        (node.value, node.down, node.right)
    }
}

// Returns an LDD containing only the given vector, i.e., { vector }
pub fn singleton(storage: &mut Storage, vector: &[u64]) -> Ldd
{
    let mut root = storage.empty_vector();
    for val in vector.iter().rev()
    {
        root = storage.insert(*val, root, storage.empty_set());
    }

    root
}

// Returns the union of the given LDDs.
pub fn union(storage: &mut Storage, a: Ldd, b: Ldd) -> Ldd
{
    if a == b {
        a
    } else if a == storage.empty_set() {
        b
    } else if b == storage.empty_set() {
        a
    } else {
        let (a_value, a_down, a_right) = storage.get(a);
        let (b_value, b_down, b_right) = storage.get(b);

        match a_value.cmp(&b_value) {
            Ordering::Less => {
                let result = union(storage, a_right, b);
                storage.insert(a_value, a_down, result)
            },
            Ordering::Equal => {
                let down_result = union(storage, a_down, b_down);
                let right_result = union(storage, a_right, b_right);
                storage.insert(a_value, down_result, right_result)
            },
            Ordering::Greater => {
                let result = union(storage, a, b_right);
                storage.insert(b_value, b_down, result)
            }
        }
    }
}

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
    if ldd == storage.empty_set() {
        Ok(())
    } 
    else if ldd == storage.empty_vector() 
    {
        // Here, we have found another vector in the LDD.
        write!(f, "<")?;
        for val in cache
        {
            write!(f, "{} ", val)?;
        }
        write!(f, ">\n")
    }
    else
    {
        // Loop over all nodes on this level
        let mut current = ldd;

        loop
        {
            let (value, down, right) = storage.get(current);

            cache.push(value);
            print(storage, cache, down, f)?;
            cache.pop();

            if right == storage.empty_set()
            {
                break
            }
            else
            {
                current = right;
            }
        }
        Ok(())        
    }
}

impl fmt::Display for Display<'_>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let mut cache: Vec<u64> = Vec::new();

        write!(f, "{{ ")?;
        print(self.storage, &mut cache, self.ldd, f)?;
        write!(f, "}}")
    }
}