use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, Policy, PolicyAnd}, AlohomoraType};
use rocket::data;
use serde::Serialize;
use mysql::prelude::Queryable;

use crate::context::ContextDataTypeOut;


#[derive(Clone, Serialize, Debug)]
pub struct CandidateDataPolicy {
    // you can only access sensitive candidate data if: 
    //      a) you are that candidate
    //      b) you are an admin
    //     ~c) you are a parent <- doesn't actually matter bc parents can't have accounts

    candidate_id: Option<i32>,
}

impl CandidateDataPolicy {
    pub fn new(candidate_id: Option<i32>) -> Self {
        CandidateDataPolicy{ candidate_id }
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
        let context: &ContextDataTypeOut = context.downcast_ref().unwrap();
        // context.conn

        return true;
        
        // // make sure we have an id and key
        // let session_id = match &context.session_id {
        //     Some(s) => s.to_owned(),
        //     None => return false,
        // };
        // let key = match &context.key {
        //     Some(s) => s.to_owned(),
        //     None => return false,
        // };

        

        
        // let mut db = context.db.lock().unwrap();
        // //let a: Conn = db.into();

        // // check if we have a valid admin session
        // let admin_sessions = db.exec_iter("SELECT * FROM admin_session WHERE id = ? AND expires_at > NOW()", (session_id.clone(),)).unwrap();
        // //                                               ^^check session id is valid    ^^it hasn't expired    
        // if admin_sessions.count() == 1 { return true; }


        // // verify that we're a valid candidate & get our id
        // let mut candidate_sessions = db.exec_iter("SELECT * FROM session WHERE id = ? AND expires_at > NOW()", (session_id,)).unwrap();

        // let my_id: i32 = match candidate_sessions.next() {
        //     None => return false,
        //     Some(row_res) => mysql::from_value(row_res.unwrap().get(1).unwrap())
        // };
        
        // // if we have a valid session, check if we're the candidate with the data
        // match self.candidate_id {
        //     None => false,
        //     Some(data_id) => my_id == data_id,
        // }
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
            CandidateDataPolicy::new(self.candidate_id)
        } else {
            // if not, no users should be allowed to access the data
            CandidateDataPolicy{candidate_id: None}
        };

        Ok(p)
    }
}

impl ORMPolicy for CandidateDataPolicy {
    fn from_result(result: &sea_orm::prelude::QueryResult) -> Self {
        let candidate_id: Option<i32> = match result.try_get_by(0) {
            Ok(id) => Some(id),
            Err(_) => None,
        };

        CandidateDataPolicy { 
            candidate_id,
        }
    }

    fn empty() -> Self where Self: Sized {
        CandidateDataPolicy{
            candidate_id: None,
        }
    }
}