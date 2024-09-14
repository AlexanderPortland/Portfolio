use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, Policy, PolicyAnd}, AlohomoraType};
use rocket::{data, figment::value::magic::Either};
use serde::Serialize;
use mysql::prelude::Queryable;

use crate::context::ContextDataTypeOut;


#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct CandidateDataPolicy {
    // you can only access sensitive candidate data (PLAIN or CIPHERTEXT) if: 
    //      a) you are that candidate
    //      b) you are an admin

    candidate_id: Option<i32>,
}

impl CandidateDataPolicy {
    pub fn new(candidate_id: Option<i32>) -> Self {
        CandidateDataPolicy{ candidate_id }
    }
}

impl Default for CandidateDataPolicy {
    fn default() -> Self {
        println!("defaulting!!");
        CandidateDataPolicy { candidate_id: None }
    }
}

impl Policy for CandidateDataPolicy {
    fn name(&self) -> String {
        match self.candidate_id {
            Some(id) => format!("Candidate Data Policy (id: {id})"),
            None => format!("Candidate Data Policy (only accessible by admins)"),
        }
    }

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        // let context: &ContextDataTypeOut = context.downcast_ref().unwrap();
        // context.conn

        return true;
    }

    fn join(&self, other: alohomora::policy::AnyPolicy) -> Result<alohomora::policy::AnyPolicy, ()> {
        if other.is::<CandidateDataPolicy>() {
            let other = other.specialize().unwrap();
            return Ok(AnyPolicy::new(self.join_logic(other)?));
        } else {
            return Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()), 
                other)
            ));
        }
    }

    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        let p = if self.candidate_id == other.candidate_id {
            // if they have the same id, keep it
            CandidateDataPolicy{ candidate_id: self.candidate_id }
        } else {
            // if not, no users should be allowed to access the data
            CandidateDataPolicy{ candidate_id: None }
        };

        Ok(p)
    }
}

impl ORMPolicy for CandidateDataPolicy {
    fn from_result(result: &sea_orm::prelude::QueryResult) -> Self {
        let candidate_id: i32 = match result.try_get("", "candidate_id") {
            Ok(r) => r,
            Err(_) => {
                // so either we are in the candidate table where it's just called `id`
                match result.try_get("", "id") {
                    Ok(r) => r,
                    // or something went wrong
                    Err(_) => panic!("issue making candidate data policy from db"),
                }
            }
        };

        CandidateDataPolicy { 
            candidate_id: Some(candidate_id),
        }
    }

    fn empty() -> Self where Self: Sized {
        CandidateDataPolicy{
            candidate_id: None,
        }
    }
}

impl FrontendPolicy for CandidateDataPolicy {
    fn from_cookie<'a, 'r>(
            name: &str,
            cookie: &'a rocket::http::Cookie<'static>,
            request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        todo!()
    }

    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
            where
                Self: Sized {
        todo!()
    }
}