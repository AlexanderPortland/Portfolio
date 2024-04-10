use alohomora::{bbox::BBox, policy::NoPolicy};
use serde::{Serialize, Deserialize};
use validator::Validate;

use crate::error::ServiceError;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct School {
    #[validate(length(min = 1, max = 255))]
    name: String,
    #[validate(length(min = 1, max = 255))]
    field: String,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct BBoxSchool {
//     name: BBox<String, NoPolicy>,
//     field: BBox<String, NoPolicy>,
// }

// impl BBoxSchool {
//     pub fn from_school(s: School) -> Self {
//         BBoxSchool {
//             name: BBox::new(s.name.clone(), NoPolicy::new()),
//             field: BBox::new(s.field.clone(), NoPolicy::new())
//         }
//     }
// }

// fn bbox_a_school(s: School) -> BBoxSchool {
//     BBoxSchool {
//         name: BBox::new(s.name.clone(), NoPolicy::new()),
//         field: BBox::new(s.field.clone(), NoPolicy::new())
//     }
// }

impl School {
    pub fn from_opt_str(school: Option<BBox<String, NoPolicy>>) -> Option<Self> {
        school.map(
            |school| serde_json::from_str(&school.discard_box()).unwrap() // TODO: handle error
        )
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