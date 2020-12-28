use std::ops::Deref;
use async_trait::async_trait;
use crate::{
    application::CONTAINER,
    guards::{
        Guard,
        GuardOutcome,
    },
    OxideRequest,
};


pub struct State<T: Send + Sync + 'static>(&'static T);

impl<T: Send + Sync + 'static> State<T> {
    pub fn into_inner(&self) -> &'static T {
        self.0
    }

    pub fn new(data: &'static T) -> Self {
        Self(data)
    }
}

impl<T: Send + Sync + 'static> Deref for State<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &'static T {
        &self.0
    }
}

#[async_trait]
impl<'a, T: Send + Sync + 'static> Guard for State<T> {
    async fn from_request(_: OxideRequest) -> GuardOutcome<Self> {
        GuardOutcome::Value(
            Self(CONTAINER.get::<T>())
        )
    }
}
