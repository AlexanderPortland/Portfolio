use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, Policy, PolicyAnd}};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyPolicy {
    pub owner: Option<String>
    // just generated, orm, or cookie

        // custom accessible everywhere
}

impl KeyPolicy {
    pub fn new(owner: Option<String>) -> Self {
        Self {
            owner
        }
    }
}

// Policy
impl Policy for KeyPolicy {
    fn name(&self) -> String {
        match self.owner.clone() {
            None => format!("KeyPolicy(for no users)"),
            Some(owner) => format!("KeyPolicy(for {}'s keys)", owner),
        }
    }

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        println!("checking key");
        // 1. check must be for setting a cookie
        let crate::Reason::Cookie(c) = reason else {
            return false;
        };

        // 2. check must be for the right owner

        // 3. check must be at the login endpoint?

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
        let (Some(own1), Some(own2)) = (self.owner.clone(), other.owner) else {
            return Ok(KeyPolicy { owner: None });
        };
        if own1.eq(&own2) {
            return Ok(KeyPolicy { owner: None });
        } else {
            return Ok(KeyPolicy { owner: Some(own1) });
        }
    }
}

// ORM Policy
impl ORMPolicy for KeyPolicy {
    fn empty() -> Self where Self: Sized {
        KeyPolicy { owner: None }
    }

    fn from_result(result: &sea_orm::QueryResult) -> Self
        where
            Self: Sized {
        println!("getting from result");
        let name: String = result.try_get("", "name").unwrap();
        // should this panic? or should this just return with None?
        println!("got result {}", name);
        KeyPolicy{
            owner: Some(name)
        }
    }
}

// optionally store duplicate of auth struct in context
impl FrontendPolicy for KeyPolicy {

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