// use alohomora::{bbox::BBox, policy::AnyPolicy, AlohomoraType};
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

pub fn serialize_app_row(rows: Vec<ApplicationRow>) -> Result<Vec<u8>, ServiceError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.into_inner().map_err(|_| ServiceError::CsvIntoInnerError)
}

pub fn serialize_cand_row(rows: Vec<CandidateRow>) -> Result<Vec<u8>, ServiceError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.into_inner().map_err(|_| ServiceError::CsvIntoInnerError)
}

// // *********** NEEDED FOR serialization sandboxes ***********

// service error
pub enum ServiceError {
    CsvIntoInnerError, // we only need this one error bc its the only one we can throw from in the sandbox
}

// application row
#[derive(Serialize)]
pub struct ApplicationRow {
    ////#[serde(rename = "Ev. č. přihlášky")]
    pub application: i32,
    //#[serde(rename = "Jméno")]
    pub name: Option<String>,
    //#[serde(rename = "Příjmení")]
    pub surname: Option<String>,
    //#[serde(rename = "Rodné příjmení (pokud odlišné)")]
    pub birth_surname: Option<String>,
    //#[serde(rename = "Místo narození")]
    pub birthplace: Option<String>,
    //#[serde(rename = "Datum narození")]
    pub birthdate: Option<String>,
    //#[serde(rename = "Adresa trvalého pobytu")]
    pub address: Option<String>,
    //#[serde(rename = "Adresa pro doručování písemností (pokud odlišné)")]
    pub letter_address: Option<String>,
    //#[serde(rename = "Telefon")]
    pub telephone: Option<String>,
    //#[serde(rename = "Státní občanství")]
    pub citizenship: Option<String>,
    //#[serde(rename = "Email")]
    pub email: Option<String>,
    //#[serde(rename = "Pohlaví")]
    pub sex: Option<String>,
    //#[serde(rename = "Rodné číslo")]
    pub personal_identification_number: Option<String>,
    //#[serde(rename = "Název školy (IZO)")]
    pub school_name: Option<String>,
    //#[serde(rename = "Zdravotní pojištění")]
    pub health_insurance: Option<String>,

    //#[serde(rename = "Vysvědčení 1/8")]
    pub diploma_1_8: String,
    //#[serde(rename = "Vysvědčení 2/8")]
    pub diploma_2_8: String,
    //#[serde(rename = "Vysvědčení 1/9")]
    pub diploma_1_9: String,
    //#[serde(rename = "Vysvědčení 2/9")]
    pub diploma_2_9: String,

    //#[serde(rename = "První škola - název")]
    pub first_school_name: Option<String>,
    //#[serde(rename = "První škola - obor")]
    pub first_school_field: Option<String>,
    //#[serde(rename = "Druhá škola - název")]
    pub second_school_name: Option<String>,
    //#[serde(rename = "Druhá škola - obor")]
    pub second_school_field: Option<String>,

    //#[serde(rename = "Jméno zákonného zástupce")]
    pub parent_name: Option<String>,
    //#[serde(rename = "Příjmení zákonného zástupce")]
    pub parent_surname: Option<String>,
    //#[serde(rename = "Telefon zákonného zástupce")]
    pub parent_telephone: Option<String>,
    //#[serde(rename = "Email zákonného zástupce")]
    pub parent_email: Option<String>,

    //#[serde(rename = "Jméno druhého zákonného zástupce")]
    pub second_parent_name: Option<String>,
    //#[serde(rename = "Příjmení druhého zákonného zástupce")]
    pub second_parent_surname: Option<String>,
    //#[serde(rename = "Telefon druhého zákonného zástupce")]
    pub second_parent_telephone: Option<String>,
    //#[serde(rename = "Email druhého zákonného zástupce")]
    pub second_parent_email: Option<String>,
}

// candidate row
#[derive(Serialize)]
pub struct CandidateRow {
    pub id: i32,
    pub first_application: i32,
    pub second_application: Option<i32>,
    pub personal_id_number: String,
    pub first_day_admissions: bool,
    pub second_day_admissions: bool,
    pub first_day_field: Option<FieldOfStudy>,
    pub second_day_field: Option<FieldOfStudy>,
    pub fields_combination: FieldsCombination,
    pub first_school: String,
    pub first_school_field: String,
    pub second_school: String,
    pub second_school_field: String,
    pub fields_match: bool,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub telephone: String,
    pub parent_email: Option<String>,
    pub parent_telephone: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum FieldOfStudy {
    G,
    IT,
    KB,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub enum FieldsCombination {
    #[serde(rename = "Žádný obor na SSPŠ")]
    Unknown,
    #[serde(rename = "G")]
    G,
    #[serde(rename = "IT")]
    IT,
    #[serde(rename = "KB")]
    KB,
    #[serde(rename = "G a IT")]
    GIt,
    #[serde(rename = "G a KB")]
    GKb,
    #[serde(rename = "IT a KB")]
    ItKb,
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