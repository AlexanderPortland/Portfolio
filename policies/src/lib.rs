use alohomora::context::UnprotectedContext;
use alohomora::orm::ORMPolicy;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy, Reason};
use rocket::http::Cookie;
use rocket::Request;
use sea_orm_migration::sea_orm::QueryResult;

pub mod candidate;
pub mod context;
pub mod key;

#[derive(Clone, Debug, PartialEq)]
pub struct FakePolicy {}

impl FakePolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Policy for FakePolicy {
    fn name(&self) -> String {
        String::from("FakePolicy")
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        println!("IN FAKE POLICY");
        true
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(FakePolicy {}))
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        Ok(FakePolicy {})
    }
}

impl FrontendPolicy for FakePolicy {
    fn from_request<'a, 'r>(request: &'a Request<'r>) -> Self where Self: Sized {
        FakePolicy {}
    }

    fn from_cookie<'a, 'r>(name: &str, cookie: &'a Cookie<'static>, request: &'a Request<'r>) -> Self where Self: Sized {
        FakePolicy {}
    }
}

impl ORMPolicy for FakePolicy {
    fn from_result(result: &QueryResult) -> Self where Self: Sized {
        FakePolicy {}
    }
    fn empty() -> Self where Self: Sized { FakePolicy {} }
}

impl Default for FakePolicy {
    fn default() -> Self {
        FakePolicy {}
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct ACLPolicy {}

impl ACLPolicy {
    pub fn new() -> Self {
        Self {}
    }
}

impl Policy for ACLPolicy {
    fn name(&self) -> String {
        String::from("ACLPolicy")
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        true
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(ACLPolicy {}))
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        Ok(ACLPolicy {})
    }
}

impl FrontendPolicy for ACLPolicy {
    fn from_request<'a, 'r>(request: &'a Request<'r>) -> Self where Self: Sized {
        ACLPolicy {}
    }

    fn from_cookie<'a, 'r>(name: &str, cookie: &'a Cookie<'static>, request: &'a Request<'r>) -> Self where Self: Sized {
        ACLPolicy {}
    }
}

impl ORMPolicy for ACLPolicy {
    fn from_result(result: &QueryResult) -> Self where Self: Sized {
        ACLPolicy {}
    }
    fn empty() -> Self where Self: Sized { ACLPolicy {} }
}

impl Default for ACLPolicy {
    fn default() -> Self {
        ACLPolicy {}
    }
}