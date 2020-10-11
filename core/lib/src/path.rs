use std::{
    str::FromStr,
    ops::{
        Deref,
        DerefMut,
    },
};


pub struct Path<T: FromStr>(T);

impl<T: FromStr> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: FromStr> DerefMut for Path<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: FromStr> From<T> for Path<T> {
    fn from(inner: T) -> Path<T> {
        Path(inner)
    }
}

impl<T: FromStr> FromStr for Path<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Path<T>, Self::Err> {
        let t = T::from_str(s)?;

        Ok(Self(t))
    }
}
