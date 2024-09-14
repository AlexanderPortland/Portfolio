use core::panic;

use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, Policy, PolicyAnd}, testing::TestContextData, AlohomoraType};
use rocket::{data, figment::value::magic::Either, State};
use sea_orm::{ConnectionTrait, Statement};
use serde::Serialize;
use mysql::prelude::Queryable;

use crate::context::ContextDataTypeOut;

#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct CandidateDataPolicy {
    session_id: Option<String>, // only set for data coming from client POST
    candidate_id: Option<i32>,  // only set for data coming from DB
}

// (->) candidate data can enter the system
//     a. as post data (we have session_id cookie) <- FrontendPolicy
//     b. as query from DB (we have candidate_id)  <- ORMPolicy

// (<-) candidate data can be leaked
//     a. as rendering (we have session_id, pk?)
//     b. into pcr (we don't care to validate)
//     c. into db  (^^)

impl CandidateDataPolicy {
    pub fn new(candidate_id: Option<i32>) -> Self {
        CandidateDataPolicy{ 
            session_id: None,
            candidate_id 
        }
    }
}

impl Default for CandidateDataPolicy {
    fn default() -> Self {
        println!("defaulting!!");
        CandidateDataPolicy { session_id: None, candidate_id: None }
    }
}

impl Policy for CandidateDataPolicy {
    fn name(&self) -> String {
        match self.candidate_id {
            Some(id) => format!("Candidate Data Policy (id: {id})"),
            None => format!("Candidate Data Policy (only accessible by admins)"),
        }
    }

    // right client (cand_id) render -> ok
    // any admin render -> ok
    // right client (in session) db -> ok
    // custom region -> okay
    // EVERYTHING ELSE -> nuh uh

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        println!("thank you sir! you've given me {:?}", context);
        

        match reason {
            // 0. we trust the custom reviewers
            alohomora::policy::Reason::Custom(_) => {
                println!("Custom reason");
                return true
            },
            // 1. if writing to DB, make sure it's from the same session as data
            alohomora::policy::Reason::DB(_, _) => {
                println!("DB reason");
                return true;
            }
            // 2. if rendering, we must either be a) an admin, or b) the right candidate
            alohomora::policy::Reason::TemplateRender(_) | alohomora::policy::Reason::Response => {
                println!("render reason for me {:?}", self);
                let context: &ContextDataTypeOut = if let Some(test) = context.downcast_ref::<TestContextData<ContextDataTypeOut>>() {
                    // test.0
                    // FIXME: how to downcast to testcontext data here
                    println!("test context data");
                    todo!()
                } else {
                    println!("real context data");
                    context.downcast_ref().unwrap()
                };

                let session_id = context.session_id.clone().unwrap();
                let session_id = sea_orm::prelude::Uuid::parse_str(session_id.as_str()).unwrap();

                println!("got it!");
                
                // if let Some(candidate_id) = self.candidate_id {
                //     // candidate check (your session_id exists for the data's candidate_id)
                //     let session_id = context.session_id.clone().unwrap();
                //     println!("session id is {session_id}");
                //     let result = rocket::tokio::task::block_in_place(||{
                //         let res = context.conn.query_all(Statement::from_string(
                //                 context.conn.get_database_backend(),
                //                 format!("select * from session where id = {} and candidate_id = {};", session_id, candidate_id),
                //             ));
                //         let result = rocket::tokio::runtime::Handle::current().block_on(res);
                //         result.unwrap()
                //     });
                //     println!("got query response {:?}", result);
                //     println!("len is {:?}", result.len());
                //     todo!();
                // }
                    // println!("session id is {session_id}");
                    // println!("session id is {}", session_id.as_u128());
                    // println!("session id is {}", session_id.as_simple());
                    // println!("session id is 0X{:X}", session_id.as_u128());

                // admin check
                let result = rocket::tokio::task::block_in_place(||{
                    let res = context.conn.query_all(Statement::from_string(
                            context.conn.get_database_backend(),
                            // format!("select * from admin_session where id = {};", session_id),
                            format!("select * from admin_session where id=0x{:x};", session_id.as_u128()),
                        ));
                    let result = rocket::tokio::runtime::Handle::current().block_on(res);
                    result.unwrap()
                });
                if result.len() >= 1 {
                    return true;
                }
                
                todo!()
                // let res = context.conn.execute(Statement::from_string(
                //     context.conn.get_database_backend(),
                //     String::from(""),
                // )).await?;
            },
            _ => {
                println!("other");
                return false
            },
        }

        return false;
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
        let (mut candidate_id, mut session_id) = (None, None);
        if self.candidate_id == other.candidate_id {
            // if they have the same id, keep it
            candidate_id = self.candidate_id;
        }
        if self.session_id == other.session_id {
            session_id = self.session_id.clone();
        }
        Ok(CandidateDataPolicy{
            candidate_id,
            session_id,
        })
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

        println!("found candidate id {candidate_id}");

        CandidateDataPolicy { 
            candidate_id: Some(candidate_id),
            session_id: None,
        }
    }

    fn empty() -> Self where Self: Sized {
        CandidateDataPolicy{
            candidate_id: None,
            session_id: None,
        }
    }
}

impl FrontendPolicy for CandidateDataPolicy {
    fn from_cookie<'a, 'r>(
            name: &str,
            cookie: &'a rocket::http::Cookie<'static>,
            request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        Self::from_request(request)
    }

    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
            where
                Self: Sized {
        match request.cookies().get("id") {
            // cookie id is a session id which maps in the sessions db table to candidate_id which is what we want
            Some(session_id) => {
                println!("yahoo i got id {session_id}");
                let session_id = Some(session_id.to_string());
                println!("(or as a string) {:?}", session_id);
                CandidateDataPolicy {
                    session_id,
                    candidate_id: None,
                }
            },
            None => {
                println!("no such luck with the id cookie strategy");
                panic!();
            }
        }
    }
}