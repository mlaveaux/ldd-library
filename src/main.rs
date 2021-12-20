use std::collections::HashMap;
use std::fmt;

// List Decision Diagrams, abbreviated LDD, are data structures that represent sets of fixed length vectors. 

// An LDD node represents a set as follows. Given a LDD node n then [[n]] is defined as:
// [[false]] = emptyset
// [[true]] = { <> } (the singleton set containing the empty vector)
// [[node(value, down, right)]] = { value x | x in [[down]] } union [[right]] (where value x is the concatenation of vectors)
#[derive(PartialEq, Eq, Hash)]
struct LddNode
{
    value: u64,
    down: usize,
    right: usize
}

// To facilitate maximal sharing of nodes an LDD points to the root node.
type Ldd = usize;

// The library that implements the maximal sharing behaviour.
struct LddLibrary
{
    index: HashMap<LddNode, usize>,
    table: Vec<LddNode>,
    height: Vec<u64>,
}

impl LddLibrary
{
    fn new() -> Self
    {
        let mut library = Self { 
            index: HashMap::new(),
            table: Vec::new(),
            height: Vec::new(),
        };

        // Add two nodes representing 'false' and 'true' respectively; these cannot be created using make_node.
        library.table.push(LddNode{
            value: 0,
            down: 0,
            right: 0,
        });
        library.table.push(LddNode{
            value: 0,
            down: 0,
            right: 0,
        });

        // Only used for debugging purposes. height(false) = 0 and height(true) = 0, note that height(false) is irrelevant
        library.height.push(0);
        library.height.push(0);
        
        library
    }

    fn make_node(&mut self, value: u64, down: Ldd, right: Ldd) -> Ldd
    {
        // Check the validity of the down and right nodes.
        assert_ne!(down, self.false_node());
        assert_ne!(right, self.true_node());
        assert!(down < self.table.len());
        assert!(right < self.table.len());

        if right != self.false_node()
        {
            // Check that our height matches the right LDD.
            assert_eq!(self.height[down] + 1, self.height[right]);
            // Check that our value is less then the right value.
            assert!(value < self.value(right));
        }

        let new_node = LddNode {value, down, right};
        *self.index.entry(new_node).or_insert_with(
            || 
            {
                self.table.push(LddNode {value, down, right});
                self.height.push(self.height[down] + 1);
                self.table.len() - 1
            }
        )
    }

    fn false_node(&self) -> Ldd
    {
        return 0
    }

    fn true_node(&self) -> Ldd
    {
        return 1
    }

    fn value(&self, ldd: Ldd) -> u64
    {
        self.table[ldd].value
    }

    fn get_node(&self, ldd: Ldd) -> &LddNode
    {
        &self.table[ldd]
    }
}

// Returns an LDD containing only the given vector, i.e., { vector }
fn singleton(library: &mut LddLibrary, vector: &[u64]) -> Ldd
{
    let mut root = library.true_node();
    for val in vector.iter().rev()
    {
        root = library.make_node(*val, root, library.false_node());
    }

    root
}

// Return a formatter for the given Ldd.
fn fmt_node(library: &LddLibrary, ldd: Ldd) -> LddDisplay
{
    LddDisplay {
        library,
        ldd,
    }
}


fn main() {

    // Initialize the library.
    let mut library = LddLibrary::new();

    let node = singleton(&mut library, &[0, 1, 2, 3, 4]);

    println! ("node {}", fmt_node(&library, node))
}


// Print the lists represented by the given LddNode.
struct LddDisplay<'a>
{
    library: &'a LddLibrary,
    ldd: Ldd,
}

fn print(library: &LddLibrary, ldd: Ldd, f: &mut fmt::Formatter<'_>) -> fmt::Result
{
    if ldd == library.false_node() {
        return write!(f, "")
    } else if ldd == library.true_node() {
        return write!(f, "]")
    }

    write!(f, "{}", library.value(ldd))
}

impl fmt::Display for LddDisplay<'_>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{{ <");
        print(self.library, self.ldd, f);
        write!(f, "> }}")
    }
}