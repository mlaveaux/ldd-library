use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

use crate::{Storage, Ldd};

use super::ldd::ProtectionSet;

/// The operation cache can significantly speed up operations by caching
/// intermediate results. This is necessary since the maximal sharing means that
/// the same inputs can be encountered many times while evaluating the
/// operations.
/// 
/// For all operations defined in `operations.rs` where caching helps we
/// introduce a cache. The cache that belongs to one operation is identified by
/// the value of [UnaryFunction], [BinaryOperator] or [TernaryOperator].
pub struct OperationCache
{
    protection_set: Rc<RefCell<ProtectionSet>>,
    caches1: Vec<FxHashMap<usize, usize>>,
    caches2: Vec<FxHashMap<(usize, usize), usize>>,
    caches3: Vec<FxHashMap<(usize, usize, usize), usize>>,
}

/// Any function from LDD -> usize.
pub enum UnaryFunction
{
    Len,
}

/// Any operator from LDD x LDD -> LDD.
pub enum BinaryOperator
{
    Union,
    Minus,
}

/// Any operator from LDD x LDD x LDD -> LDD.
pub enum TernaryOperator
{
    RelationalProduct,
}

impl OperationCache
{
    pub fn new(protection_set: Rc<RefCell<ProtectionSet>>) -> OperationCache
    {
        OperationCache {
            protection_set,
            caches1: vec![FxHashMap::default()],
            caches2: vec![FxHashMap::default(); 2],
            caches3: vec![FxHashMap::default()],
        }
    }

    /// Clear all existing caches. This must be done during garbage collection
    /// since caches have references to elements in the node table that are not
    /// protected.
    pub fn clear(&mut self)
    {    
        for cache in self.caches1.iter_mut() {
            cache.clear();
        }

        for cache in self.caches2.iter_mut() {
            cache.clear();
        }

        for cache in self.caches3.iter_mut() {
            cache.clear();
        }    
    }

    fn get_cache1(&mut self, operator: &UnaryFunction) -> &mut FxHashMap<usize, usize>
    {
        match operator {
            UnaryFunction::Len => &mut self.caches1[0],
        }
    }

    fn get_cache2(&mut self, operator: &BinaryOperator) -> &mut FxHashMap<(usize, usize), usize>
    {
        match operator {
            BinaryOperator::Union => &mut self.caches2[0],
            BinaryOperator::Minus => &mut self.caches2[1]
        }
    }

    fn get_cache3(&mut self, operator: &TernaryOperator) -> &mut FxHashMap<(usize, usize, usize), usize>
    {
        match operator {
            TernaryOperator::RelationalProduct => &mut self.caches3[0],
        }
    }

    /// Create an Ldd from the given index. Only safe because this is a private function.
    fn create(&mut self, index: usize) -> Ldd
    {
        Ldd::new(&self.protection_set, index)
    }
}


/// Implements an operation cache for a unary LDD operator.
pub fn cache_unary_function<F>(storage: &mut Storage, operator: UnaryFunction, a: &Ldd, f: F) -> usize
    where F: Fn(&mut Storage, &Ldd) -> usize
{
    let key = a.index();
    if let Some(result) = storage.operation_cache().get_cache1(&operator).get(&key) 
    {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        result
    }
    else 
    {
        let result = f(storage,  a);
        storage.operation_cache().get_cache1(&operator).insert(key, result);
        result
    }
}

/// Implements an operation cache for a binary LDD operator.
pub fn cache_binary_op<F>(storage: &mut Storage, operator: BinaryOperator, a: Ldd, b: Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, Ldd, Ldd) -> Ldd
{
    let key = (a.index(), b.index());
    if let Some(result) = storage.operation_cache().get_cache2(&operator).get(&key) 
    {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    }
    else 
    {
        let result = f(storage,  a, b);
        storage.operation_cache().get_cache2(&operator).insert(key, result.index());
        result
    }
}

/// Implements an operation cache for a commutative binary LDD operator, i.e.,
/// an operator f such that f(a,b) = f(b,a) for all LDD a and b.
pub fn cache_comm_binary_op<F>(storage: &mut Storage, operator: BinaryOperator, a: Ldd, b: Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, Ldd, Ldd) -> Ldd
{
    // Reorder the inputs to improve caching behaviour (can potentially half the cache size)
    if a.index() < b.index() {
        cache_binary_op(storage, operator, a, b, f)
    } else {
        cache_binary_op(storage, operator, b, a, f)
    }
}

/// Implements an operation cache for a terniary LDD operator.
pub fn cache_terniary_op<F>(storage: &mut Storage, operator: TernaryOperator, a: &Ldd, b: &Ldd, c: &Ldd, f: F) -> Ldd
    where F: Fn(&mut Storage, &Ldd, &Ldd, &Ldd) -> Ldd
{
    let key = (a.index(), b.index(), c.index());
    if let Some(result) = storage.operation_cache().get_cache3(&operator).get(&key) 
    {
        let result = *result; // Necessary to decouple borrow from storage and the call to create.
        storage.operation_cache().create(result)
    }
    else 
    {
        let result = f(storage,  a, b, c);
        storage.operation_cache().get_cache3(&operator).insert(key, result.index());
        result
    }
}