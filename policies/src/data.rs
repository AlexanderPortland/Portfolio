use core::panic;

use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, NoPolicy, Policy, PolicyAnd}, testing::TestContextData, AlohomoraType};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::Serialize;

use crate::context::ContextDataTypeOut;

#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct CandidateDataPolicy {
    // only set for data coming from client POST
    session_id: Option<String>, 
    
    // only set for data coming from DB
    candidate_id: Option<i32>,    // (candidate table)
    application_id: Option<i32>,  // (application table)
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
            candidate_id,
            application_id: None,
        }
    }
}

impl Default for CandidateDataPolicy {
    fn default() -> Self {
        println!("defaulting!!");
        CandidateDataPolicy { session_id: None, candidate_id: None, application_id: None }
    }
}

fn does_session_exist(is_admin: bool, db: &DatabaseConnection, session_id: String, candidate_id: Option<i32>, application_id: Option<i32>) -> bool {
    let application_id = if let Some(id) = application_id { 
        // println!("already have app id {id}");
        Some(id) 
    } else if let Some(candidate_id) = candidate_id {
        // println!("trying to get app id for cand {candidate_id}");
        let result = rocket::tokio::task::block_in_place(||{
            let res = db.query_all(Statement::from_string(
                    db.get_database_backend(),
                    // format!("select * from admin_session where id = {};", session_id),
                    format!("select * from application where candidate_id={candidate_id};"),
                ));
            rocket::tokio::runtime::Handle::current().block_on(res).unwrap()
        }).first().unwrap().try_get::<i32>("", "id");
        // println!("result {:?}", result);
        match result {
            Ok(ok) => Some(ok),
            Err(_) => None
        }
    } else {
        None
    };

    println!("seeing if session exists w/ as admin {is_admin}, session_id {session_id}, candidate_id: {:?}, application_id: {:?}", candidate_id, application_id);
    let session_id = sea_orm::prelude::Uuid::parse_str(session_id.as_str()).unwrap();
    let table_name = if is_admin { String::from("admin_session") } else { String::from("session") };
    let id_phrase = if let Some(application_id) = application_id {
        format!(" and candidate_id = {}", application_id)
    } else { String::from("") };
    let result = rocket::tokio::task::block_in_place(||{
        let res = db.query_all(Statement::from_string(
                db.get_database_backend(),
                // format!("select * from admin_session where id = {};", session_id),
                format!("select * from {table_name} where id=0x{:x} {id_phrase};", session_id.as_u128()),
            ));
        rocket::tokio::runtime::Handle::current().block_on(res).unwrap()
    });
    
    // if candidate_id.is_some() {
    //     println!("res is {:?}", result);
    //     println!("id should be {:?}", result.get(0).unwrap().try_get::<i32>("", "candidate_id"));
    //     // todo!()
    // }
    result.len() >= 1
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
        // println!("thank you sir! you've given me {:?}", context);
        

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
                // println!("render reason for me {:?}", self);
                let context: &ContextDataTypeOut = if let Some(test) = context.downcast_ref::<TestContextData<ContextDataTypeOut>>() {
                    // test.0
                    // FIXME: how to downcast to testcontext data here
                    // println!("test context data");
                    todo!()
                } else {
                    context.downcast_ref().unwrap()
                };
                // println!("real context data {:?}", context);

                let session_id = context.session_id.clone().unwrap();
                let session_id = sea_orm::prelude::Uuid::parse_str(session_id.as_str()).unwrap();

                println!("got it!");

                // admin check
                if does_session_exist(true, &context.conn, context.session_id.clone().unwrap(), None, None) {
                    return true;
                }

                // candidate (same session return result)
                if let Some(session_id) = self.session_id.clone() {
                    println!("cand same session check");
                    return session_id == context.session_id.clone().unwrap();
                }
                // candidate check
                if let Some(_) = self.candidate_id {
                    println!("from cand check");
                    return does_session_exist(false, &context.conn, context.session_id.clone().unwrap(), self.candidate_id, None);
                }
                return false
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
            println!("data stacking polciies w/ other {:?}", other);
            if other == AnyPolicy::new(NoPolicy::new()){ // TODO: why do I need this??
                return Ok(AnyPolicy::new(self.clone()));
            }
            return Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()), 
                other)
            ));
        }
    }

    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        let (mut candidate_id, mut session_id, mut application_id) = (None, None, None);
        if self.candidate_id == other.candidate_id {
            // if they have the same id, keep it
            candidate_id = self.candidate_id;
        }
        if self.session_id == other.session_id {
            session_id = self.session_id.clone();
        }
        if self.application_id == other.application_id {
            application_id = self.application_id.clone();
        }
        Ok(CandidateDataPolicy{
            candidate_id,
            session_id,
            application_id
        })
    }
}

impl ORMPolicy for CandidateDataPolicy {
    fn from_result(result: &sea_orm::prelude::QueryResult) -> Self {
        let policy = if let Ok(r) = result.try_get::<i32>("", "ip_address") {
            // in session table
            println!("in session table w/ result {:?}", r);
            todo!()
        } else if let Ok(r) = result.try_get::<i32>("", "password") {
            // in application table
            println!("in application table w/ result {:?}", r);
            match result.try_get::<i32>("", "id") {
                Ok(application_id) => CandidateDataPolicy { 
                    application_id: Some(application_id),
                    session_id: None,
                    candidate_id: None,
                },
                Err(e) => panic!("{:?}", e),
            }
        } else if let Ok(r) = result.try_get::<i32>("", "candidate_id") {
            // in parent table
            println!("in parent table w/ result {:?}", r);
            CandidateDataPolicy { 
                candidate_id: Some(r),
                session_id: None,
                application_id: None,
            }
        } else {
            // in the candidate table table
            println!("in candidate table");
            match result.try_get("", "id") {
                Ok(candidate_id) => CandidateDataPolicy { 
                    candidate_id: Some(candidate_id),
                    session_id: None,
                    application_id: None,
                },
                // or something went wrong
                Err(e) => panic!("issue making candidate data policy from db {e}"),
            }
        };

        println!("found policy {:?}", policy);
        policy
    }

    fn empty() -> Self where Self: Sized {
        CandidateDataPolicy{
            candidate_id: None,
            application_id: None,
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
        println!("in route {}", request.uri());
        if request.uri() == "/candidate/login" || request.route().unwrap().to_string() == "" {
            println!("special route");
        } else {
            println!("unspecial route");
        }
        match request.cookies().get("id") {
            // cookie id is a session id which maps in the sessions db table to candidate_id which is what we want
            Some(session_id) => {
                println!("yahoo i got id {session_id}");
                let session_id = Some(session_id.value().to_string());
                println!("(or as a string) {:?}", session_id);
                CandidateDataPolicy {
                    session_id,
                    candidate_id: None,
                    application_id: None,
                }
            },
            None => {
                // legally won't have cookie set yet on login
                if request.uri() == "/candidate/login" {
                    CandidateDataPolicy {
                        session_id: None,
                        candidate_id: None,
                        application_id: None,
                    }
                } else {
                    println!("no candidate id cookie at endpoint that needs it");
                    panic!();
                }
            }
        }
    }
}