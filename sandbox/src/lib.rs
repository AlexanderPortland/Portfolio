use alohomora_derive::AlohomoraSandbox;
use portfolio_types::*;

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

#[AlohomoraSandbox()]
fn serde_from_grade(t: GradeList) -> String {
    serde_json::to_string(&t).unwrap()
}

#[AlohomoraSandbox()]
fn serde_to_school(t: String) -> School {
    serde_json::from_str(t.as_str()).unwrap()
}

#[AlohomoraSandbox()]
fn serde_to_grade(t: String) -> GradeList {
    serde_json::from_str(t.as_str()).unwrap()
}

type Tup = (GradeList, GradeList, GradeList, GradeList);
#[AlohomoraSandbox()]
pub fn serde_from_tuple((t, i): (Tup, u8)) -> String {
    match i {
        0 => serde_json::to_string(&t.0).unwrap(),
        1 => serde_json::to_string(&t.1).unwrap(),
        2 => serde_json::to_string(&t.2).unwrap(),
        3 => serde_json::to_string(&t.3).unwrap(),
        _ => panic!("invalid i")
    }
}

#[AlohomoraSandbox()]
pub fn serialize_app_row(rows: Vec<ApplicationRow>) -> Result<Vec<u8>, ServiceError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.into_inner().map_err(|_| ServiceError::CsvIntoInnerError)
}

#[AlohomoraSandbox()]
pub fn serialize_cand_row(rows: Vec<CandidateRow>) -> Result<Vec<u8>, ServiceError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.into_inner().map_err(|_| ServiceError::CsvIntoInnerError)
}