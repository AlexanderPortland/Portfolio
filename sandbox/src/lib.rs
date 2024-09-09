use alohomora::sandbox::AlohomoraSandbox;
use serde::{Deserialize, Serialize};

extern crate alohomora;
// extern crate chrono;

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

// #[AlohomoraSandbox]
// fn naive_date_str((date, format): (chrono::NaiveDate, bool)) -> String {
//     match format {
//         true => date.to_string(),
//         false => date.format(NAIVE_DATE_FMT).to_string(),
//     }
// }

#[AlohomoraSandbox]
fn serde_from_grade(t: GradeList) -> String {
    serde_json::to_string(&t).unwrap()
}

#[AlohomoraSandbox]
fn serde_from_school(t: School) -> String {
    serde_json::to_string(&t).unwrap()
}

#[AlohomoraSandbox]
fn serde_to_school(t: String) -> School {
    serde_json::from_str(t.as_str()).unwrap()
}

#[AlohomoraSandbox]
fn serde_to_grade(t: String) -> GradeList {
    serde_json::from_str(t.as_str()).unwrap()
}

// ************** NEEDED FOR school sandboxes **************

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct School {
    name: String,
    field: String,
}

// ************** NEEDED FOR grade sandboxes **************

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeList(Vec<Grade>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grade {
    subject: String,
    semester: Semester,
    value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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