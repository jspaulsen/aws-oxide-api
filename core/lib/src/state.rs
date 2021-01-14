use std::ops::Deref;
use async_trait::async_trait;
use crate::{
    guards::{
        Guard,
        GuardOutcome,
    },
    RouteRequest,
};


pub struct State<'r, T: Send + Sync + 'static>(&'r T);


impl<'r, T: Send + Sync + 'static> State<'r, T> {
    pub fn new(data: &'r T) -> Self {
        Self(data)
    }
}

impl<'r, T: Send + Sync> Deref for State<'r, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.0
    }
}

#[async_trait]
impl<'a, 'r, T: Send + Sync + 'static> Guard<'a, 'r> for State<'r, T> {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<State<'r, T>> {
        let data = request
            .container
            .get::<T>()
            .clone();

        GuardOutcome::Value(Self::new(data))
    }
}
