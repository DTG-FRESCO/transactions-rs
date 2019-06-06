use std::{collections::HashSet, hash::Hash};

use lending_library::{LendingLibrary, Loan};

pub struct LendingWrap<'a, K, V>
where
    K: Eq + Hash,
{
    inner: &'a mut LendingLibrary<K, V>,
    added: LendingLibrary<K, V>,
    removed: HashSet<K>,
}

impl<'a, K, V> LendingWrap<'a, K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(lib: &'a mut LendingLibrary<K, V>) -> Self {
        LendingWrap {
            inner: lib,
            added: LendingLibrary::new(),
            removed: HashSet::new(),
        }
    }

    pub fn commit(self) {
        for k in self.removed {
            self.inner.remove(&k);
        }
        self.inner.extend(self.added);
    }

    pub fn rollback(self) {}

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.added.contains_key(&k) {
            self.added.insert(k, v)
        } else {
            if self.removed.contains(&k) {
                self.removed.remove(&k);
                self.added.insert(k, v)
            } else {
                if self.inner.contains_key(&k) {
                    let item = self.inner.lend(&k).unwrap();
                    let ret = Some((*item).clone());
                    self.added.insert(k, v);
                    ret
                } else {
                    self.added.insert(k, v);
                    None
                }
            }
        }
    }

    pub fn contains_key(&self, k: &K) -> bool {
        !self.removed.contains(k) && (self.added.contains_key(k) || self.inner.contains_key(k))
    }

    pub fn remove(&mut self, k: &K) -> bool {
        if self.added.contains_key(k) {
            self.removed.insert(k.clone());
            self.added.remove(k)
        } else {
            if self.removed.contains(k) {
                false
            } else {
                self.removed.insert(k.clone());
                self.inner.contains_key(k)
            }
        }
    }

    pub fn lend(&mut self, k: &K) -> Option<Loan<K, V>> {
        if self.added.contains_key(k) {
            self.added.lend(k)
        } else {
            if self.removed.contains(k) {
                None
            } else {
                if self.inner.contains_key(k) {
                    let item = self.inner.lend(&k).unwrap();
                    self.added.insert(k.clone(), (*item).clone());
                    self.added.lend(k)
                } else {
                    None
                }
            }
        }
    }
}
