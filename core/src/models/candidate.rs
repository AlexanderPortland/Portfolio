use std::collections::HashMap;
use alohomora::policy::AnyPolicy;
use alohomora::{bbox::BBox, policy::NoPolicy, AlohomoraType};
use alohomora::rocket::ResponseBBoxJson;
use alohomora_derive::RequestBBoxJson;
use chrono::NaiveDate;
use entity::{application, candidate};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::ServiceError;

use super::{candidate_details::{EncryptedString, EncryptedCandidateDetails}, grade::GradeList, school::School};

#[derive(Debug, Clone, Serialize)]
pub enum FieldOfStudy {
    G,
    IT,
    KB,
}

impl Into<String> for FieldOfStudy {
    fn into(self) -> String {
        match self {
            FieldOfStudy::G => "G".to_string(),
            FieldOfStudy::IT => "IT".to_string(),
            FieldOfStudy::KB => "KB".to_string(),
        }
    }
}

impl From<i32> for FieldOfStudy {
    fn from(id: i32) -> Self {
        match &id.to_string().as_str()[0..3] {
            "101" => FieldOfStudy::G,
            "102" => FieldOfStudy::IT,
            "103" => FieldOfStudy::KB,
            _ => panic!("Invalid field of study id"), // TODO: handle using TryFrom
        }
    }
}

impl TryFrom<String> for FieldOfStudy {
    type Error = ServiceError;
    fn try_from(s: String) -> Result<Self, ServiceError> {
        match s.as_str() {
            "7941K41-Gymnázium" => Ok(FieldOfStudy::G),
            "1820M01-Informační technologie" => Ok(FieldOfStudy::IT), // TODO: constants
            "1820M01-Informační technologie - Kybernetická bezpečnost" => Ok(FieldOfStudy::KB),
            _ => Err(ServiceError::InvalidFieldOfStudy),
        }
    }
}

impl Into<i32> for FieldOfStudy {
    fn into(self) -> i32 {
        match self {
            FieldOfStudy::G => 101,
            FieldOfStudy::IT => 102,
            FieldOfStudy::KB => 103,
        }
    }
}

/// Minimal candidate response containing database only not null fields
#[derive(ResponseBBoxJson, AlohomoraType)]
pub struct NewCandidateResponse {
    pub current_application: BBox<i32, AnyPolicy>,
    pub applications: Vec<BBox<i32, NoPolicy>>,
    pub personal_id_number: BBox<String, NoPolicy>,
    pub details_filled: BBox<bool, NoPolicy>,
    pub encrypted_by: Option<BBox<i32, NoPolicy>>,
    pub field_of_study: BBox<String, NoPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanNewCandidateResponse {
    pub current_application: i32,
    pub applications: Vec<i32>,
    pub personal_id_number: String,
    pub details_filled: bool,
    pub encrypted_by: Option<i32>,
    pub field_of_study: String,
}

/// Create candidate (admin endpoint)
/// Password change  (admin endpoint)
#[derive(Debug, ResponseBBoxJson)]
pub struct CreateCandidateResponse {
    pub application_id: BBox<i32, NoPolicy>,
    pub field_of_study: BBox<String, NoPolicy>,
    pub applications: Vec<BBox<i32, NoPolicy>>,
    pub personal_id_number: BBox<String, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanCreateCandidateResponse {
    pub application_id: i32,
    pub field_of_study: String,
    pub applications: Vec<i32>,
    pub personal_id_number: String,
    pub password: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, ResponseBBoxJson, RequestBBoxJson)]
pub struct CandidateDetails {
    pub name: BBox<String, NoPolicy>,
    pub surname: BBox<String, NoPolicy>,
    pub birthSurname: BBox<String, NoPolicy>,
    pub birthplace: BBox<String, NoPolicy>,
    pub birthdate: BBox<NaiveDate, NoPolicy>,
    pub address: BBox<String, NoPolicy>,
    pub letterAddress: BBox<String, NoPolicy>,
    pub telephone: BBox<String, NoPolicy>,
    pub citizenship: BBox<String, NoPolicy>,
    pub email: BBox<String, NoPolicy>,
    pub sex: BBox<String, NoPolicy>,
    pub personalIdNumber: BBox<String, NoPolicy>,
    pub schoolName: BBox<String, NoPolicy>,
    pub healthInsurance: BBox<String, NoPolicy>,
    pub grades: BBox<GradeList, NoPolicy>,
    pub firstSchool: BBox<School, NoPolicy>,
    pub secondSchool: BBox<School, NoPolicy>,
    pub testLanguage: BBox<String, NoPolicy>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CleanCandidateDetails {
    pub name: String,
    pub surname: String,
    pub birth_surname: String,
    pub birthplace: String,
    pub birthdate: NaiveDate,
    pub address: String,
    pub letter_address: String,
    pub telephone: String,
    pub citizenship: String,
    #[validate(email)]
    pub email: String,
    pub sex: String,
    pub personal_id_number: String,
    pub school_name: String,
    pub health_insurance: String,
    pub grades: GradeList,
    pub first_school: School,
    pub second_school: School,
    pub test_language: String,
}

impl CandidateDetails {
    pub fn validate_self(&self) -> Result<(), ServiceError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, alohomora_derive::ResponseBBoxJson, RequestBBoxJson)]
pub struct ParentDetails {
    pub name: BBox<String, NoPolicy>,
    pub surname: BBox<String, NoPolicy>,
    pub telephone: BBox<String, NoPolicy>,
    pub email: BBox<String, NoPolicy>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CleanParentDetails {
    pub name: String,
    pub surname: String,
    pub telephone: String,
    pub email: String,
}

#[derive(Debug, Clone, ResponseBBoxJson, RequestBBoxJson)]
pub struct ApplicationDetails {
    pub candidate: CandidateDetails,
    pub parents: Vec<ParentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanApplicationDetails {
    pub candidate: CleanCandidateDetails,
    pub parents: Vec<CleanParentDetails>,
}

impl NewCandidateResponse {
    pub async fn from_encrypted(
        current_application: BBox<i32, NoPolicy>,
        applications: Vec<application::Model>,
        private_key: &BBox<String, NoPolicy>,
        c: candidate::Model,
    ) -> Result<Self, ServiceError> {
        let field_of_study = BBox::new(FieldOfStudy::from(current_application.clone().discard_box()).into(), NoPolicy::new());
        let id_number = BBox::new(EncryptedString::from(c.personal_identification_number.to_owned().discard_box())
            .decrypt(&private_key.clone().discard_box())
            .await?, NoPolicy::new());
        let applications = applications.iter().map(|a| a.id.clone()).collect::<Vec<BBox<i32, NoPolicy>>>();
        let encrypted_details = EncryptedCandidateDetails::from(&c);

        Ok(Self {
            current_application: current_application.into_any_policy(),
            applications,
            personal_id_number: id_number,
            details_filled: BBox::new(encrypted_details.is_filled(), NoPolicy::new()),
            encrypted_by: c.encrypted_by_id,
            field_of_study,
        })
    }
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

impl FieldsCombination {
    pub fn from_fields(first: &Option<FieldOfStudy>, second: &Option<FieldOfStudy>) -> Self {
        match (first, second) {
            (None, None) => FieldsCombination::Unknown,
            (Some(FieldOfStudy::G), None) => FieldsCombination::G,
            (Some(FieldOfStudy::IT), None) => FieldsCombination::IT,
            (Some(FieldOfStudy::KB), None) => FieldsCombination::KB,
            (None, Some(FieldOfStudy::G)) => FieldsCombination::G,
            (None, Some(FieldOfStudy::IT)) => FieldsCombination::IT,
            (None, Some(FieldOfStudy::KB)) => FieldsCombination::KB,
            // Field combinations
            (Some(FieldOfStudy::G), Some(FieldOfStudy::IT)) => FieldsCombination::GIt,
            (Some(FieldOfStudy::G), Some(FieldOfStudy::KB)) => FieldsCombination::GKb,
            (Some(FieldOfStudy::IT), Some(FieldOfStudy::KB)) => FieldsCombination::ItKb,
            (Some(FieldOfStudy::IT), Some(FieldOfStudy::G)) => FieldsCombination::GIt,
            (Some(FieldOfStudy::KB), Some(FieldOfStudy::G)) => FieldsCombination::GKb,
            (Some(FieldOfStudy::KB), Some(FieldOfStudy::IT)) => FieldsCombination::ItKb,
            // Some candidates filled in the same field twice
            (Some(FieldOfStudy::G), Some(FieldOfStudy::G)) => FieldsCombination::G,
            (Some(FieldOfStudy::IT), Some(FieldOfStudy::IT)) => FieldsCombination::IT,
            (Some(FieldOfStudy::KB), Some(FieldOfStudy::KB)) => FieldsCombination::KB,
        }
    }
}

#[derive(AlohomoraType)]
#[alohomora_out_type(to_derive = [Serialize])]
pub struct CandidateRow {
    pub id: BBox<i32, NoPolicy>,
    pub first_application: BBox<i32, NoPolicy>,
    pub second_application: BBox<Option<i32>, NoPolicy>,
    pub personal_id_number: BBox<String, NoPolicy>,
    pub first_day_admissions: BBox<bool, NoPolicy>,
    pub second_day_admissions: BBox<bool, NoPolicy>,
    pub first_day_field: BBox<Option<FieldOfStudy>, NoPolicy>,
    pub second_day_field: BBox<Option<FieldOfStudy>, NoPolicy>,
    pub fields_combination: BBox<FieldsCombination, NoPolicy>,
    pub first_school: BBox<String, NoPolicy>,
    pub first_school_field: BBox<String, NoPolicy>,
    pub second_school: BBox<String, NoPolicy>,
    pub second_school_field: BBox<String, NoPolicy>,
    pub fields_match: BBox<bool, NoPolicy>,
    pub name: BBox<String, NoPolicy>,
    pub surname: BBox<String, NoPolicy>,
    pub email: BBox<String, NoPolicy>,
    pub telephone: BBox<String, NoPolicy>,
    pub parent_email: BBox<Option<String>, NoPolicy>,
    pub parent_telephone: BBox<Option<String>, NoPolicy>,
}