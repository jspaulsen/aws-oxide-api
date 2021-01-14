use std::{
    ops::{
        Deref,
        DerefMut,
    },
};
use async_trait::async_trait;
use crate::{
    Body,
    guards::{
        Guard,
        GuardOutcome,
    },
    request::RouteRequest,
};


pub struct Binary(Vec<u8>);

pub struct Text(String);


impl Binary {
    pub fn new(t: Vec<u8>) -> Self {
        Self(t)
    }
}

impl Text {
    pub fn new(t: String) -> Self {
        Self(t)
    }
}


impl Deref for Binary {
    type Target = Vec<u8>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Binary {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Vec<u8> {
        &mut self.0
    }
}

impl Into<Vec<u8>> for Binary {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl Deref for Text {
    type Target = String;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Text {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl Into<String> for Text {
    fn into(self) -> String {
        self.0
    }
}

#[async_trait]
impl<'a, 'r> Guard<'a, 'r> for Binary {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<Self> {
        match request.body() {
            Body::Binary(body) => {
                GuardOutcome::Value(
                    Self::new(
                        body.clone()
                    )
                )
            },
            _ => GuardOutcome::Forward
        }
    }
}

#[async_trait]
impl<'a, 'r> Guard<'a, 'r> for Text {
    async fn from_request(request: &'a RouteRequest<'r>) -> GuardOutcome<Self> {
        match request.body() {
            Body::Text(body) => {
                GuardOutcome::Value(
                    Self::new(
                        body.clone()
                    )
                )
            },
            _ => GuardOutcome::Forward
        }
    }
}
