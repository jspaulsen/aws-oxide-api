use std::ops::Deref;
use async_trait::async_trait;
use crate::{
    guards::{
        Guard,
        GuardOutcome,
    },
    OxideRequest,
};


pub struct State<T: Send + Sync>(T);


impl<T: Send + Sync> State<T> {
    pub fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: Send + Sync + Clone> Deref for State<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

#[async_trait]
impl<'a, T: Send + Sync + Clone + 'static> Guard for State<T> {
    async fn from_request(request: OxideRequest) -> GuardOutcome<Self> {
        let data = request
            .container
            .get::<T>()
            .clone();

        GuardOutcome::Value(Self::new(data))
    }
}
