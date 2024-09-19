use alohomora::context::UnprotectedContext;
use alohomora::orm::ORMPolicy;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy, Reason};
use rocket::http::Cookie;
use rocket::Request;
use sea_orm_migration::sea_orm::QueryResult;

pub mod data;
pub mod context;
pub mod key;
pub mod request;

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
    fn check(&self, _: &UnprotectedContext, _: Reason<'_>) -> bool {
        // println!("IN FAKE POLICY");
        true
    }
    fn join(&self, _: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(FakePolicy {}))
    }
    fn join_logic(&self, _: Self) -> Result<Self, ()> where Self: Sized {
        Ok(FakePolicy {})
    }
}

impl FrontendPolicy for FakePolicy {
    fn from_request<'a, 'r>(_: &'a Request<'r>) -> Self where Self: Sized {
        FakePolicy {}
    }

    fn from_cookie<'a, 'r>(_: &str, _: &'a Cookie<'static>, _: &'a Request<'r>) -> Self where Self: Sized {
        FakePolicy {}
    }
}

impl ORMPolicy for FakePolicy {
    fn from_result(_: &QueryResult) -> Self where Self: Sized {
        FakePolicy {}
    }
    fn empty() -> Self where Self: Sized { FakePolicy {} }
}

impl Default for FakePolicy {
    fn default() -> Self {
        FakePolicy {}
    }
}