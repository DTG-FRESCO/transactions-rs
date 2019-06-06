use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    ops::Index,
};

use hashlike::HashLike;
pub mod commit_behavior {
    mod sealed {
        use super::*;
        pub trait Sealed {}
        impl Sealed for PanicIfUnfinalised {}
        impl Sealed for ImplicitRollback {}
        impl Sealed for ImplicitCommit {}
    }
    pub trait Behavior: sealed::Sealed {}
    pub struct PanicIfUnfinalised;
    pub struct ImplicitRollback;
    pub struct ImplicitCommit;
    impl Behavior for PanicIfUnfinalised {}
    impl Behavior for ImplicitRollback {}
    impl Behavior for ImplicitCommit {}
}

#[derive(Debug)]
pub struct HashWrap<'a, K, V, T = HashMap<K, V>, B = commit_behavior::PanicIfUnfinalised>
where
    HashWrap<'a, K, V, T, B>: SpecDrop,
    K: Eq + Hash,
    T: HashLike<K, V>,
    B: commit_behavior::Behavior,
{
    inner: &'a mut T,
    added: HashMap<K, V>,
    removed: HashSet<K>,
    commit_behaviour: PhantomData<B>,
    finalised: bool,
}

impl<'a, K, V, T, B> HashWrap<'a, K, V, T, B>
where
    HashWrap<'a, K, V, T, B>: SpecDrop,
    K: Eq + Hash,
    T: HashLike<K, V>,
    B: commit_behavior::Behavior,
{
    pub fn new(map: &'a mut T) -> Self {
        HashWrap {
            inner: map,
            added: HashMap::new(),
            removed: HashSet::new(),
            commit_behaviour: PhantomData,
            finalised: false,
        }
    }

    fn _commit(&mut self) {
        for k in &self.removed {
            self.inner.remove(&k);
        }
        for (k, v) in self.added.drain() {
            self.inner.insert(k, v);
        }
        self.finalised = true;
    }

    pub fn commit(mut self) {
        self._commit()
    }

    fn _rollback(&mut self) {
        self.finalised = true;
    }

    pub fn rollback(mut self) {
        self._rollback()
    }

    pub fn contains_key(&self, k: &K) -> bool {
        !self.removed.contains(k) && (self.added.contains_key(k) || self.inner.contains_key(k))
    }
}

impl<'a, K, V, T> HashWrap<'a, K, V, T>
where
    K: Eq + Hash + Clone,
    T: HashLike<K, V>,
    V: Clone,
{
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.added.contains_key(&k) {
            self.added.insert(k, v)
        } else {
            if self.removed.contains(&k) {
                self.removed.remove(&k);
                self.added.insert(k, v)
            } else {
                if self.inner.contains_key(&k) {
                    let ret = Some(self.inner.get(&k).unwrap().clone());
                    self.added.insert(k, v);
                    ret
                } else {
                    self.added.insert(k, v);
                    None
                }
            }
        }
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        if self.added.contains_key(k) {
            self.removed.insert(k.clone());
            self.added.remove(k)
        } else {
            if self.removed.contains(k) {
                None
            } else {
                self.removed.insert(k.clone());
                if self.inner.contains_key(k) {
                    Some(self.inner.get(k).unwrap().clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        if self.added.contains_key(k) {
            self.added.get_mut(k)
        } else {
            if self.removed.contains(k) {
                None
            } else {
                if self.inner.contains_key(k) {
                    self.added
                        .insert(k.clone(), self.inner.get(k).unwrap().clone());
                    self.added.get_mut(k)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a, 'b, K, V, T> Index<&'b K> for HashWrap<'a, K, V, T>
where
    K: Eq + Hash + Clone,
    T: HashLike<K, V>,
    V: Clone,
{
    type Output = V;

    fn index(&self, index: &'b K) -> &Self::Output {
        if self.added.contains_key(index) {
            self.added.index(index)
        } else {
            if self.removed.contains(index) {
                panic!()
            } else {
                self.inner.get(index).unwrap()
            }
        }
    }
}

pub trait SpecDrop {
    fn spec_drop(&mut self);
}

impl<'a, K, V, T> SpecDrop for HashWrap<'a, K, V, T, commit_behavior::PanicIfUnfinalised>
where
    K: Eq + Hash,
    T: HashLike<K, V>,
{
    fn spec_drop(&mut self) {
        panic!("Error: Dropping wrapper without calling commit or rollback.")
    }
}

impl<'a, K, V, T> SpecDrop for HashWrap<'a, K, V, T, commit_behavior::ImplicitCommit>
where
    K: Eq + Hash,
    T: HashLike<K, V>,
{
    fn spec_drop(&mut self) {
        self._commit();
    }
}

impl<'a, K, V, T> SpecDrop for HashWrap<'a, K, V, T, commit_behavior::ImplicitRollback>
where
    K: Eq + Hash,
    T: HashLike<K, V>,
{
    fn spec_drop(&mut self) {
        self._rollback();
    }
}

impl<'a, K, V, T, B> Drop for HashWrap<'a, K, V, T, B>
where
    HashWrap<'a, K, V, T, B>: SpecDrop,
    K: Eq + Hash,
    T: HashLike<K, V>,
    B: commit_behavior::Behavior,
{
    fn drop(&mut self) {
        if !self.finalised {
            self.spec_drop();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_hash() -> HashMap<i32, String> {
        let mut h = HashMap::new();
        h.insert(0, "Zero".to_string());
        h.insert(1, "One".to_string());
        h.insert(2, "Two".to_string());
        h
    }

    fn check_hash(val: HashMap<i32, String>) {
        let r = get_hash();
        assert_eq!(val.len(), r.len());
        for (k, v) in val {
            assert_eq!(v, r[&k]);
        }
    }

    #[test]
    fn basic_shadowing() {
        let mut map = get_hash();
        let wrap = HashWrap::new(&mut map);
        assert_eq!(wrap[&1], "One");
        wrap.commit();
        check_hash(map);
    }

    #[test]
    fn basic_rollback() {
        let mut map = get_hash();
        let mut wrap = HashWrap::new(&mut map);
        assert_eq!(wrap[&2], "Two");
        let ret = wrap.insert(5, "Five".to_string());
        assert!(ret.is_none());
        assert_eq!(wrap[&5], "Five");
        let ret = wrap.remove(&0);
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "Zero");
        wrap.rollback();
        check_hash(map);
    }

    #[test]
    fn basic_edits() {
        let mut map = get_hash();
        let mut wrap = HashWrap::new(&mut map);
        let ret = wrap.insert(5, "Five".to_string());
        assert!(ret.is_none());
        wrap.get_mut(&2).unwrap().push_str("00");
        wrap.commit();
        assert!(map.contains_key(&5));
        assert_eq!(map[&2], "Two00");
    }

    #[test]
    fn basic_removal() {
        let mut map = get_hash();
        let mut wrap = HashWrap::new(&mut map);
        let ret = wrap.remove(&1);
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "One");
        wrap.commit();
        assert!(!map.contains_key(&1));
    }

    #[test]
    fn repeated_insertion() {
        let mut map = get_hash();
        let mut wrap = HashWrap::new(&mut map);
        let ret = wrap.insert(1, "Five".to_string());
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "One");
        let ret = wrap.insert(1, "Three".to_string());
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "Five");
        let ret = wrap.insert(1, "Four".to_string());
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "Three");
        let ret = wrap.remove(&1);
        assert!(ret.is_some());
        assert_eq!(ret.unwrap(), "Four");
        wrap.rollback();
        check_hash(map);
    }
}
