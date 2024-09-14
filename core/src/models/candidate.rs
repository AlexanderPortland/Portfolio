use std::collections::HashMap;
use alohomora::policy::AnyPolicy;
use alohomora::{bbox::BBox, AlohomoraType};
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::ResponseBBoxJson;
use chrono::NaiveDate;
use alohomora::policy::Policy;
use entity::{application, candidate};
use portfolio_policies::data::CandidateDataPolicy;
use serde::{Deserialize, Serialize};
use validator::Validate;
use portfolio_policies::FakePolicy;

use crate::error::ServiceError;

use super::{candidate_details::{EncryptedString, EncryptedCandidateDetails}, grade::GradeList, school::School};

#[derive(Debug, Clone, Serialize)]
pub enum FieldOfStudy {
    G,
    IT,
    KB,
}

fn from_option(o: Option<FieldOfStudy>) -> Option<portfolio_sandbox::FieldOfStudy>{
    match o {
        None => None,
        Some(b) => { Some(b.into()) }
    }
}

// convert to sandbox equivalent
impl From<FieldOfStudy> for portfolio_sandbox::FieldOfStudy {
    fn from(value: FieldOfStudy) -> Self {
        match value {
            FieldOfStudy::G => portfolio_sandbox::FieldOfStudy::G,
            FieldOfStudy::IT => portfolio_sandbox::FieldOfStudy::IT,
            FieldOfStudy::KB => portfolio_sandbox::FieldOfStudy::KB,
            _ => todo!()
        }
    }
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
    pub applications: Vec<BBox<i32, AnyPolicy>>,
    pub personal_id_number: BBox<String, AnyPolicy>,
    pub details_filled: bool,
    pub encrypted_by: Option<BBox<i32, AnyPolicy>>,
    pub field_of_study: BBox<String, AnyPolicy>,
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
    pub application_id: BBox<i32, AnyPolicy>,
    pub field_of_study: BBox<String, AnyPolicy>,
    pub applications: Vec<BBox<i32, AnyPolicy>>,
    pub personal_id_number: BBox<String, AnyPolicy>,
    pub password: BBox<String, AnyPolicy>,
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
#[derive(Debug, Clone, PartialEq, ResponseBBoxJson)]
pub struct CandidateDetails {
    pub name: BBox<String, AnyPolicy>,
    pub surname: BBox<String, AnyPolicy>,
    pub birthSurname: BBox<String, AnyPolicy>,
    pub birthplace: BBox<String, AnyPolicy>,
    pub birthdate: BBox<NaiveDate, AnyPolicy>,
    pub address: BBox<String, AnyPolicy>,
    pub letterAddress: BBox<String, AnyPolicy>,
    pub telephone: BBox<String, AnyPolicy>,
    pub citizenship: BBox<String, AnyPolicy>,
    pub email: BBox<String, AnyPolicy>,
    pub sex: BBox<String, AnyPolicy>,
    pub personalIdNumber: BBox<String, AnyPolicy>,
    pub schoolName: BBox<String, AnyPolicy>,
    pub healthInsurance: BBox<String, AnyPolicy>,
    pub grades: BBox<GradeList, AnyPolicy>,
    pub firstSchool: BBox<School, AnyPolicy>,
    pub secondSchool: BBox<School, AnyPolicy>,
    pub testLanguage: BBox<String, AnyPolicy>,
}

#[derive(Debug, Clone, PartialEq, ResponseBBoxJson)]
pub struct ParentDetails {
    pub name: BBox<String, AnyPolicy>,
    pub surname: BBox<String, AnyPolicy>,
    pub telephone: BBox<String, AnyPolicy>,
    pub email: BBox<String, AnyPolicy>,
}

#[derive(Debug, Clone, ResponseBBoxJson)]
pub struct ApplicationDetails {
    pub candidate: CandidateDetails,
    pub parents: Vec<ParentDetails>,
}

impl NewCandidateResponse {
    pub async fn from_encrypted<P: Policy + Clone + 'static>(
        current_application: BBox<i32, CandidateDataPolicy>,
        applications: Vec<application::Model>,
        private_key: &BBox<String, P>,
        c: candidate::Model,
    ) -> Result<Self, ServiceError> {
        let field_of_study = current_application.clone().into_ppr(
            PrivacyPureRegion::new(|id: i32| {
                FieldOfStudy::from(id).into()
            })
        );
        let personal_id_number = EncryptedString::from(c.personal_identification_number.clone())
            .decrypt(private_key)
            .await?;

        let applications = applications.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
        let encrypted_details = EncryptedCandidateDetails::from(&c);

        Ok(Self {
            current_application: current_application.into_any_policy(),
            applications: applications.into_iter().map(|b| b.into_any_policy()).collect(),
            personal_id_number: personal_id_number,
            details_filled: encrypted_details.is_filled(),
            encrypted_by: c.encrypted_by_id.map(|b| b.into_any_policy()),
            field_of_study: field_of_study.into_any_policy(),
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

fn from_combo(b: FieldsCombination) -> portfolio_sandbox::FieldsCombination{
    b.into()
}

// convert to sandbox equivalent
impl From<FieldsCombination> for portfolio_sandbox::FieldsCombination {
    fn from(value: FieldsCombination) -> Self {
        match value {
            FieldsCombination::Unknown => portfolio_sandbox::FieldsCombination::Unknown,
            FieldsCombination::G => portfolio_sandbox::FieldsCombination::G,
            FieldsCombination::IT => portfolio_sandbox::FieldsCombination::IT,
            FieldsCombination::KB => portfolio_sandbox::FieldsCombination::KB,
            FieldsCombination::GIt => portfolio_sandbox::FieldsCombination::GIt,
            FieldsCombination::GKb => portfolio_sandbox::FieldsCombination::GKb,
            FieldsCombination::ItKb => portfolio_sandbox::FieldsCombination::ItKb,
            _ => todo!()
        }
    }
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
    pub id: BBox<i32, AnyPolicy>,
    pub first_application: BBox<i32, AnyPolicy>,
    pub second_application: Option<BBox<i32, AnyPolicy>>,
    pub personal_id_number: BBox<String, AnyPolicy>,
    pub first_day_admissions: BBox<bool, AnyPolicy>,
    pub second_day_admissions: BBox<bool, AnyPolicy>,
    pub first_day_field: Option<BBox<FieldOfStudy, AnyPolicy>>,
    pub second_day_field: Option<BBox<FieldOfStudy, AnyPolicy>>,
    pub fields_combination: BBox<FieldsCombination, AnyPolicy>,
    pub first_school: BBox<String, AnyPolicy>,
    pub first_school_field: BBox<String, AnyPolicy>,
    pub second_school: BBox<String, AnyPolicy>,
    pub second_school_field: BBox<String, AnyPolicy>,
    pub fields_match: BBox<bool, AnyPolicy>,
    pub name: BBox<String, AnyPolicy>,
    pub surname: BBox<String, AnyPolicy>,
    pub email: BBox<String, AnyPolicy>,
    pub telephone: BBox<String, AnyPolicy>,
    pub parent_email: Option<BBox<String, AnyPolicy>>,
    pub parent_telephone: Option<BBox<String, AnyPolicy>>,
}

impl From<CandidateRowOut> for portfolio_sandbox::CandidateRow {
    fn from(value: CandidateRowOut) -> Self {
        portfolio_sandbox::CandidateRow { id: value.id, first_application: value.first_application, second_application: value.second_application, personal_id_number: value.personal_id_number, first_day_admissions: value.first_day_admissions, second_day_admissions: value.second_day_admissions, first_day_field: from_option(value.first_day_field), second_day_field: from_option(value.second_day_field), fields_combination: from_combo(value.fields_combination), first_school: value.first_school, first_school_field: value.first_school_field, second_school: value.second_school, second_school_field: value.second_school_field, fields_match: value.fields_match, name: value.name, surname: value.surname, email: value.email, telephone: value.telephone, parent_email: value.parent_email, parent_telephone: value.parent_telephone }
    }
}