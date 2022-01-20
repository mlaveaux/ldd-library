use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ldd::*;
use rand::Rng;
use std::collections::HashSet;

/// Returns a vector of the given length with random u64 values (from 0..max_value).
pub fn random_vector(length: u64, max_value: u64) -> Vec<u64> 
{
    let mut rng = rand::thread_rng();    
    let mut vector: Vec<u64> = Vec::new();
    for _ in 0..length
    {
        vector.push(rng.gen_range(0..max_value));
    }

    vector
}


/// Returns a set of 'amount' vectors where every vector has the given length.
pub fn random_vector_set(amount: u64, length: u64, max_value: u64) ->  HashSet<Vec<u64>>
{
    let mut result: HashSet<Vec<u64>> = HashSet::new();

    // Insert 'amount' number of vectors into the result.
    for _ in 0..amount
    {
        result.insert(random_vector(length, max_value));
    }

    result
}

/// Returns an LDD containing all elements of the given iterator over vectors.
pub fn from_iter<'a, I>(storage: &mut Storage, iter: I) -> Ldd
    where I: Iterator<Item = &'a Vec<u64>>
{
    let mut result = storage.empty_set().clone();

    for vector in iter
    {
        let single = singleton(storage, vector);
        result = union(storage, &result, &single);
    }

    result
}

pub fn criterion_benchmark(c: &mut Criterion) 
{    
    let mut storage = Storage::new();

    c.bench_function("union 1000", 
    |bencher| 
        {
            bencher.iter(
            || {
                let set_a = random_vector_set(1000, 10, 10);
                let set_b = random_vector_set(1000, 10, 10);
            
                let a = from_iter(&mut storage, set_a.iter());
                let b = from_iter(&mut storage, set_b.iter());
            
                black_box(union(&mut storage, &a, &b));
            })
        });
}
 	
mod perf;

#[cfg(pprof)]
criterion_group!(benches, criterion_benchmark(Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100))));

#[cfg(not(pprof))]
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);