use alohomora::sandbox::execute_sandbox;
use alohomora::{bbox::BBox};
use alohomora::policy::Policy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{OutputBBoxValue, ResponseBBoxJson};
use csv::DeserializeError;
use serde::{Serialize, Deserialize};
use validator::Validate;

use crate::error::ServiceError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Semester {
    #[serde(rename = "1/8")]
    FirstEighth,
    #[serde(rename = "2/8")]
    SecondEighth,
    #[serde(rename = "1/9")]
    FirstNinth,
    #[serde(rename = "2/9")]
    SecondNinth,
}

impl Semester {
    pub fn from_str(semester: &str) -> Result<Self, ServiceError> {
        match semester {
            "1/8" => Ok(Semester::FirstEighth),
            "2/8" => Ok(Semester::SecondEighth),
            "1/9" => Ok(Semester::FirstNinth),
            "2/9" => Ok(Semester::SecondNinth),
            _ => Err(ServiceError::FormatError)
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Semester::FirstEighth => "1/8",
            Semester::SecondEighth => "2/8",
            Semester::FirstNinth => "1/9",
            Semester::SecondNinth => "2/9",
        }
    }

    pub fn to_sandbox(self) -> portfolio_types::Semester {
        match self {
            Self::FirstEighth => portfolio_types::Semester::FirstEighth,
            Self::SecondEighth => portfolio_types::Semester::SecondEighth,
            Self::FirstNinth => portfolio_types::Semester::FirstNinth,
            Self::SecondNinth => portfolio_types::Semester::SecondNinth,
        }
    }

    pub fn from_sandbox(s: portfolio_types::Semester) -> Self {
        match s {
            portfolio_types::Semester::FirstEighth => Self::FirstEighth,
            portfolio_types::Semester::SecondEighth => Self::SecondEighth,
            portfolio_types::Semester::FirstNinth => Self::FirstNinth,
            portfolio_types::Semester::SecondNinth => Self::SecondNinth,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct Grade {
    #[validate(length(min = 1, max = 255))]
    subject: String,
    semester: Semester,
    #[validate(range(min = 1, max = 5))]
    value: i32,
}


impl Grade {
    pub fn validate_self(&self) -> Result<(), ServiceError> {
        self.validate()
            .map_err(ServiceError::ValidationError)
    }

    pub fn to_sandbox(self) -> portfolio_types::Grade {
        portfolio_types::Grade{
            subject: self.subject,
            semester: self.semester.to_sandbox(),
            value: self.value,
        }
    }

    pub fn from_sandbox(g: portfolio_types::Grade) -> Self {
        Self {
            subject: g.subject,
            semester: Semester::from_sandbox(g.semester),
            value: g.value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GradeList(Vec<Grade>);

impl ResponseBBoxJson for GradeList {
    fn to_json(self) -> OutputBBoxValue {
        OutputBBoxValue::Value(serde_json::to_value(self).unwrap())
    }
}

pub fn serde_to_grade_caller<P: Policy + Clone + 'static>(t: BBox<String, P>) -> BBox<GradeList, P> {
    let s: BBox<portfolio_types::GradeList, P> = execute_sandbox::<portfolio_sandbox::serde_to_grade, _, _>(t).specialize_policy().unwrap();

    s.into_ppr(PrivacyPureRegion::new(|gl: portfolio_types::GradeList|{
        GradeList::from_sandbox(gl)
    }))
}

// fn serde_to_grade_sandbox(t: String) -> GradeList {
//     serde_json::from_str(t.as_str()).unwrap()
// }

impl GradeList {
    pub fn validate_self(&self) -> Result<(), ServiceError> {
        self.0.iter()
            .map(|grade| grade.validate_self())
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }

    pub fn from_opt_str<P: Policy + Clone + 'static>(grades: Option<BBox<String, P>>) -> Option<BBox<Self, P>> {
        match grades {
            None => None,
            Some(grades) => Some(serde_to_grade_caller(grades)),
        }
    }

    pub fn group_by_semester(&self) -> Result<(GradeList, GradeList, GradeList, GradeList), ServiceError> {
        let mut first_semester = GradeList::default();
        let mut second_semester = GradeList::default();
        let mut third_semester = GradeList::default();
        let mut fourth_semester = GradeList::default();

        for grade in &self.0 {
            match grade.semester.as_str() {
                "1/8" => first_semester.0.push(grade.clone()),
                "2/8" => second_semester.0.push(grade.clone()),
                "1/9" => third_semester.0.push(grade.clone()),
                "2/9" => fourth_semester.0.push(grade.clone()),
                _ => return Err(ServiceError::FormatError),
            }
        }

        Ok(
            (first_semester, second_semester, third_semester, fourth_semester)
        )
    }

    pub fn to_sandbox(self) -> portfolio_types::GradeList {
        let l = self.0.into_iter().map(|g|{
            g.to_sandbox()
        }).collect::<Vec<portfolio_types::Grade>>();
        portfolio_types::GradeList(l)
    }

    pub fn from_sandbox(gl: portfolio_types::GradeList) -> Self {
        let l = gl.0.into_iter().map(|g|{
            Grade::from_sandbox(g)
        }).collect::<Vec<Grade>>();
        GradeList(l)
    }
}

impl Default for GradeList {
    fn default() -> Self {
        Self(vec![])
    }
}

impl From<Vec<Grade>> for GradeList {
    fn from(grades: Vec<Grade>) -> Self {
        Self(grades)
    }
}

impl ToString for GradeList {
    fn to_string(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }
}