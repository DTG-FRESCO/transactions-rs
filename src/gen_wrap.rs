use std::{
    mem,
    ops::{Deref, DerefMut},
};

pub struct GenericWrap<'a, T>
    where
        T: Clone,
{
    inner: &'a mut T,
    copy: Option<T>,
}

impl<'a, T> GenericWrap<'a, T>
    where
        T: Clone
{
    pub fn new(val: &'a mut T) -> Self {
        GenericWrap {
            inner: val,
            copy: None,
        }
    }

    pub fn replace(val: Self) -> Option<T> {
        let copy = val.copy;
        let inner = val.inner;
        copy.map(|v| mem::replace(inner, v))
    }

    pub fn discard(val: Self) -> Option<T> {
        val.copy
    }
}

impl<'a, T> Deref for GenericWrap<'a, T>
    where
        T: Clone
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.copy {
            Some(v) => v,
            None => &self.inner,
        }
    }
}

impl<'a, T> DerefMut for GenericWrap<'a, T>
    where
        T: Clone
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.copy {
            Some(v) => v,
            c @ None => {
                mem::replace(c, Some(self.inner.clone()));
                c.as_mut().unwrap()
            },
        }
    }
}