use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

use crate::{Storage, Ldd};

use super::ldd::ProtectionSet;

#[derive(Default)]
pub struct Cache3
{
    index: FxHashMap<(usize, usize, usize), usize>,
}

/// A cache for operators with three LDDs as input.
impl Cache3
{
    fn clear(&mut self)
    {
        self.index.clear();
    }
}

/// A cache for operators with two LDDs as input.
#[derive(Default)]
pub struct Cache2
{
    index: FxHashMap<(usize, usize), usize>,
}

impl Cache2
{
    fn clear(&mut self)
    {
        self.index.clear();
    }
}

pub struct OperationCache
{
    protection_set: Rc<RefCell<ProtectionSet>>,
    caches2: Vec<Cache2>,
    caches3: Vec<Cache3>,
}

pub enum BinaryOperator
{
    Union,
}

impl OperationCache
{
    pub fn new(protection_set: Rc<RefCell<ProtectionSet>>) -> OperationCache
    {
        OperationCache {
            protection_set,
            caches2: vec![Cache2::default(), Cache2::default()],
            caches3: vec![Cache3::default()],
        }
    }

    pub fn get_cache3(&mut self, index: usize) -> &mut Cache3
    {
        &mut self.caches3[index]
    }

    pub fn get_cache2(&mut self, index: usize) -> &mut Cache2
    {
        &mut self.caches2[index]
    }

    pub fn clear(&mut self)
    {    
        for cache in self.caches3.iter_mut() {
            cache.clear();
        }
    
        for cache in self.caches2.iter_mut() {
            cache.clear();
        }
    }

    pub fn create(&mut self, index: usize) -> Ldd
    {
        Ldd::new(&self.protection_set, index)
    }
}

/// Implements an operation cache for a terniary LDD operator.
pub fn cache_terniary_op<F>(storage: &mut Storage, operator: usize, a: &Ldd, b: &Ldd, c: &Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, &Ldd, &Ldd, &Ldd) -> Ldd
{
    let key = (a.index(), b.index(), c.index());
    if let Some(result) = storage.operation_cache().get_cache3(operator).index.get(&key) 
    {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    }
    else 
    {
        let result = f(storage,  a, b, c);
        storage.operation_cache().get_cache3(operator).index.insert(key, result.index());
        result
    }
}

/// Implements an operation cache for a binary LDD operator.
pub fn cache_binary_op<F>(storage: &mut Storage, operator: usize, a: Ldd, b: Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, Ldd, Ldd) -> Ldd
{
    let key = (a.index(), b.index());
    if let Some(result) = storage.operation_cache().get_cache2(operator).index.get(&key) 
    {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    }
    else 
    {
        let result = f(storage,  a, b);
        storage.operation_cache().get_cache2(operator).index.insert(key, result.index());
        result
    }
}

/// Implements an operation cache for a commutative binary LDD operator, i.e.,
/// an operator f such that f(a,b) = f(b,a) for all LDD a and b.
pub fn cache_comm_binary_op<F>(storage: &mut Storage, operator: usize, a: Ldd, b: Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, Ldd, Ldd) -> Ldd
{
    // Reorder the inputs to improve caching behaviour (can potentially half the cache size)
    if a.index() < b.index() {
        cache_binary_op(storage, operator, a, b, f)
    } else {
        cache_binary_op(storage, operator, b, a, f)
    }
}