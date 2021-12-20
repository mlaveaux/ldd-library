mod ldd;

fn main() {

    // Initialize the library.
    let mut storage = ldd::Storage::new();

    let node = ldd::singleton(&mut storage, &[0, 1, 2, 3, 4]);

    println! ("node {}", ldd::fmt_node(&storage, node))
}