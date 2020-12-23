use serde::de::DeserializeOwned;
use std::{
    ops::{
        Deref,
        DerefMut,
    },
};



pub struct Json<T: DeserializeOwned>(T);

impl<T: DeserializeOwned> Json<T> {
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T: DeserializeOwned> Deref for Json<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: DeserializeOwned> DerefMut for Json<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
