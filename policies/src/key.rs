use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, Policy, PolicyAnd}, AlohomoraType};

use crate::context::ContextDataTypeOut;

#[derive(Clone, Debug, PartialEq)]
pub enum KeySource {
    // Database(Option<String>), // owner_id
    Database,
    Cookie,
    JustGenerated,
}

// [for DB case checking right OWNER]
// (->) with data we have
//     - person's id
// (<-) with request we have
// * login data (AdminLoginRequest | LoginRequest)
//     - adminId & password
//     - applicationId & password


// KeyPolicy use:
// 1. when coming from DB...
//    - only goes to COOKIE of right OWNER through right ENDPOINT
// 2. when coming from user...
//    - only used in custom
// 3. after being generated...
//    - only to db

#[derive(Clone, Debug, PartialEq)]
pub struct KeyPolicy {
    pub owner_id: Option<String>, // either adminId or applicationId!!
    pub source: KeySource,
    // pub auth: Option<()>
    // just generated, orm, or cookie

        // custom accessible everywhere
}

impl KeyPolicy {
    pub fn new(id: Option<String>, source: KeySource) -> Self {
        Self {
            owner_id: id,
            source,
        }
    }
}

// Policy
impl Policy for KeyPolicy {
    fn name(&self) -> String {
        format!("{:?}", self)
    }

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        // println!("checking key policy w/ owner {:?}", self.owner_id);

        match self.source {
            // 1. if coming from db -> should only go to cookie for right person
            KeySource::Database => {
                let spec_context: &ContextDataTypeOut = context.downcast_ref().unwrap();
                println!("have context {:?}", spec_context);
                
                // 1a. it's for a key cookie
                let crate::Reason::Cookie(c) = reason else {
                    println!("NOT A COOKIE");
                    return false;
                };
                // todo!();
                const COOKIE_NAME: &str = "key";
                println!("for cookie {}", c);
                if c.ne(COOKIE_NAME) {
                    println!("NOT FOR THE {COOKIE_NAME} COOKIE");
                    return false;
                }

                // 1b. TODO: right owner
                // check to make sure login post data contains the right applicationId
                if let Some(req) = spec_context.admin_login.clone() {
                    if req.adminId.to_string() != self.owner_id.clone().unwrap() {
                        println!("req id {} isnt my owner id {}", req.adminId.to_string(), self.owner_id.clone().unwrap());
                        return false; 
                    } else {
                        println!("req id {} is my owner id {}! so you chill", req.adminId.to_string(), self.owner_id.clone().unwrap());
                    }
                }
                if let Some(req) = spec_context.candidate_login.clone() {
                    if req.applicationId.to_string() != self.owner_id.clone().unwrap() {
                        println!("req id {} isnt my owner id {}", req.applicationId.to_string(), self.owner_id.clone().unwrap());
                        todo!();
                        return false; 
                    } else {
                        println!("req id {} is my owner id {}! so you chill", req.applicationId.to_string(), self.owner_id.clone().unwrap());
                    }
                }

                // todo!();
                // (potentially) check db to make sure password matches password hash

                // 1c. login endpoint
                if context.route != "/candidate/login" && context.route != "/admin/login" {
                    println!("{} is not a chill route", context.route);
                    // todo!();
                    return false;
                }
            },
            // 2. if coming from cookie -> should only go to critical regions
            KeySource::Cookie => {
                if let crate::Reason::Custom(_) = reason {
                    // all custom sinks are chill
                    return true;
                } else {
                    return false;
                }
            },
            // 3. if coming from just generated -> should only go to db
            KeySource::JustGenerated => {
                if let crate::Reason::DB(_, _) = reason {
                    // TODO: specialize db sink??
                    return true;
                } else {
                    return false;
                }
            }
            _ => todo!(),
        }

        return true;
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<KeyPolicy>() {
            // Policies are combinable
            let other = other.specialize::<KeyPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {
            println!("stacking polciies");
            //Policies must be stacked
            Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()),
                other,
            )))
        }
    }

    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}

// ORM Policy
impl ORMPolicy for KeyPolicy {
    fn empty() -> Self where Self: Sized {
        // KeyPolicy { owner: None }
        todo!()
    }

    fn from_result(result: &sea_orm::QueryResult) -> Self
        where
            Self: Sized {
        println!("getting from result for ORM POLICy");
        let id: i32 = result.try_get("", "id").unwrap();
        // should this panic? or should this just return with None?
        println!("got id {}", id);
        KeyPolicy{
            owner_id: Some(id.to_string()),
            source: KeySource::Database,
        }
    }
}

// optionally store duplicate of auth struct in context
impl FrontendPolicy for KeyPolicy {
    fn from_cookie<'a, 'r>(
            name: &str,
            cookie: &'a rocket::http::Cookie<'static>,
            request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        Self::from_request(request)
    }
    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
            where
                Self: Sized {
        let id = request.cookies().get("id").unwrap().to_string();
        KeyPolicy { owner_id: Some(id), source: KeySource::Cookie }
    }
}

// Frontend Policy (I don't think we actually need this bc it should be applied on backend only)
// impl FrontendPolicy for KeyPolicy {
//     fn from_cookie<'a, 'r>(
//             name: &str,
//             cookie: &'a rocket::http::Cookie<'static>,
//             request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        
//     }

//     fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
//             where
//                 Self: Sized {
        
//     }
// }


// impl Policy for KeyPolicy {
//     fn name(&self) -> String {
//         String::from("KeyPolicy")
//     }
//     fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
//         true
//     }
//     fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
//         Ok(AnyPolicy::new(KeyPolicy {}))
//     }
//     fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
//         Ok(KeyPolicy {})
//     }
// }

// impl FrontendPolicy for KeyPolicy {
//     fn from_request<'a, 'r>(request: &'a Request<'r>) -> Self where Self: Sized {
//         KeyPolicy {}
//     }

//     fn from_cookie<'a, 'r>(name: &str, cookie: &'a Cookie<'static>, request: &'a Request<'r>) -> Self where Self: Sized {
//         KeyPolicy {}
//     }
// }

// impl ORMPolicy for KeyPolicy {
//     fn from_result(result: &QueryResult) -> Self where Self: Sized {
//         KeyPolicy {}
//     }
//     fn empty() -> Self where Self: Sized { KeyPolicy {} }
// }

// impl Default for KeyPolicy {
//     fn default() -> Self {
//         KeyPolicy {}
//     }
// }