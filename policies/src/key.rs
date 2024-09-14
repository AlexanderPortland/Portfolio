use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, Policy, PolicyAnd}};

#[derive(Clone, Debug, PartialEq)]
pub enum KeySource {
    Database,
    Cookie,
    JustGenerated,
}

// pub enum AnyAuth {
//     Application(ApplicationAuth),
//     Admin(AdminAuth),
// }

#[derive(Clone, Debug, PartialEq)]
pub struct KeyPolicy {
    pub owner_id: Option<String>,
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
        println!("checking key policy");

        match self.source {
            // 1. if coming from db -> should only go to cookie for right person
            KeySource::Database => {
                // 1a. it's for a key cookie
                let crate::Reason::Cookie(c) = reason else {
                    println!("NOT A COOKIE");
                    return false;
                };
                println!("for cookie {}", c);
                if c.ne("key") {
                    println!("NOT FOR THE KEY COOKIE");
                    return false;
                }

                // 1b. TODO: right owner

                // 1c. login endpoint
                if (context.route != "/candidate/login" && context.route != "/admin/login") {
                    println!("{} is not a chill route", context.route);
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
                let id = request.cookies().get("id").unwrap().to_string();
                KeyPolicy { owner_id: Some(id), source: KeySource::Cookie }
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