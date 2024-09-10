// use alohomora::sandbox::AlohomoraSandbox;
use alohomora_derive::AlohomoraSandbox;
use serde::{Deserialize, Serialize};

// extern crate alohomora;
// extern crate chrono;

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

#[AlohomoraSandbox]
fn naive_date_str((date, format): (chrono::NaiveDate, bool)) -> String {
    println!("in da sandbox w naive {:?}", date);
    match format {
        true => date.to_string(),
        false => date.format(NAIVE_DATE_FMT).to_string(),
    }
}

#[AlohomoraSandbox()]
fn serde_from_grade(t: GradeList) -> String {
    println!("in da sandbox from grade");
    serde_json::to_string(&t).unwrap()
}

#[AlohomoraSandbox()]
fn serde_from_school(t: School) -> String {
    println!("in da sandbox from school");
    serde_json::to_string(&t).unwrap()
}

#[AlohomoraSandbox()]
fn serde_to_school(t: String) -> School {
    println!("in da sandbox to school");
    serde_json::from_str(t.as_str()).unwrap()
}

#[AlohomoraSandbox()]
fn serde_to_grade(t: String) -> GradeList {
    println!("in da sandbox to grade");
    serde_json::from_str(t.as_str()).unwrap()
}

#[AlohomoraSandbox()]
fn serde_from_school_field((s, f): (School, u8)) -> String {
    println!("in da sandbox for from school field");
    match f {
        0 => s.name().to_string(),
        1 => s.field().to_string(),
        _ => panic!("invalid field"),
    }
}

type Tup = (GradeList, GradeList, GradeList, GradeList);
#[AlohomoraSandbox()]
pub fn serde_from_tuple((t, i): (Tup, u8)) -> String {
    println!("in da sandbox for from tuple");
    match i {
        0 => serde_json::to_string(&t.0).unwrap(),
        1 => serde_json::to_string(&t.1).unwrap(),
        2 => serde_json::to_string(&t.2).unwrap(),
        3 => serde_json::to_string(&t.3).unwrap(),
        _ => panic!("invalid i")
    }
}

// ************** NEEDED FOR school sandboxes **************

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct School {
    pub name: String,
    pub field: String,
}

impl School {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field(&self) -> &str {
        &self.field
    }
}

// ************** NEEDED FOR grade sandboxes **************

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeList(pub Vec<Grade>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grade {
    pub subject: String,
    pub semester: Semester,
    pub value: i32,
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