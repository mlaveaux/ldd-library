extern crate ldd;

fn main() {

    // Initialize the library.
    let mut storage = ldd::Storage::new();

    let a = ldd::singleton(&mut storage, &[0, 1, 2, 3, 4]);
    let b = ldd::singleton(&mut storage, &[0, 4, 2, 1, 8]);
    let result = ldd::union(&mut storage, a, b);

    println! ("result {}", ldd::fmt_node(&storage, result))
}