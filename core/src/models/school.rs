use std::collections::HashMap;
use alohomora::{bbox::BBox, policy::Policy};
use alohomora::pure::PrivacyPureRegion;
use serde::{Serialize, Deserialize};
use validator::Validate;

use crate::error::ServiceError;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Eq, alohomora_derive::ResponseBBoxJson)]
pub struct School {
    #[validate(length(min = 1, max = 255))]
    name: String,
    #[validate(length(min = 1, max = 255))]
    field: String,
}

pub fn serde_to_school_caller<P: Policy>(t: BBox<String, P>) -> BBox<School, P> {
    t.into_ppr(PrivacyPureRegion::new(|t|{
        serde_to_school_sandbox(t)
    }))
}

fn serde_to_school_sandbox(t: String) -> School {
    serde_json::from_str(t.as_str()).unwrap()
}

impl School {
    pub fn from_opt_str<P: Policy>(school: Option<BBox<String, P>>) -> Option<BBox<Self, P>> {
        match school {
            None => None,
            Some(school) => Some(serde_to_school_caller(school)),
        }
    }

    pub fn validate_self(&self) -> Result<(), ServiceError> {
        self.validate()
            .map_err(ServiceError::ValidationError)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

impl ToString for School {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl Default for School {
    fn default() -> Self {
        Self {
            name: String::default(),
            field: String::default(),
        }
    }
}