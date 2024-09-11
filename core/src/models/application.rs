use alohomora::{bbox::BBox, AlohomoraType};
use alohomora_derive::ResponseBBoxJson;
use chrono::NaiveDateTime;
//use sea_orm::sea_query::private;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use alohomora::policy::{AnyPolicy, NoPolicy};
use portfolio_policies::FakePolicy;

use crate::{database::query::application::ApplicationCandidateJoin, error::ServiceError};

use super::candidate_details::EncryptedString;

//#[derive(Debug, Serialize, Deserialize)]
#[derive(ResponseBBoxJson)]
pub struct ApplicationResponse {
    pub application_id: BBox<i32, AnyPolicy>,
    pub candidate_id: BBox<i32, AnyPolicy>,
    pub related_applications: Vec<BBox<i32, AnyPolicy>>,
    pub personal_id_number: BBox<String, AnyPolicy>,
    pub name: BBox<String, AnyPolicy>,
    pub surname: BBox<String, AnyPolicy>,
    pub email: BBox<String, AnyPolicy>,
    pub telephone: BBox<String, AnyPolicy>,
    pub field_of_study: Option<BBox<String, AnyPolicy>>,
    pub created_at: BBox<NaiveDateTime, AnyPolicy>,
}

#[derive(Debug, Deserialize)]
pub struct CleanApplicationResponse {
    pub application_id: i32,
    pub candidate_id: i32,
    pub related_applications: Vec<i32>,
    pub personal_id_number: String,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub telephone: String,
    pub field_of_study: Option<String>,
    pub created_at: NaiveDateTime,
}

impl ApplicationResponse {
    pub async fn from_encrypted(
        private_key: &BBox<String, FakePolicy>,
        c: ApplicationCandidateJoin,
        related_applications: Vec<BBox<i32, FakePolicy>>,
    ) -> Result<Self, ServiceError> {
        let default = BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()));

        let personal_id_number = EncryptedString::from(c.personal_id_number.to_owned()).decrypt(private_key).await?;
        let name = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.name).ok(), private_key).await?;
        let surname = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.surname).ok(), private_key).await?;
        let email = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.email).ok(), private_key).await?;
        let telephone = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.telephone).ok(), private_key).await?;
        Ok(
            Self {
                application_id: c.application_id.into_any_policy(),
                candidate_id: c.candidate_id.into_any_policy(),
                related_applications: related_applications.into_iter().map(|b| b.into_any_policy()).collect(),
                personal_id_number,
                name: name.unwrap_or(default.clone()),
                surname: surname.unwrap_or(default.clone()),
                email: email.unwrap_or(default.clone()),
                telephone:  telephone.unwrap_or(default),
                field_of_study: c.field_of_study.map(|b| b.into_any_policy()),
                created_at: c.created_at.into_any_policy(),
            }
        )
    }
}

/// CSV export (admin endpoint)
#[derive(AlohomoraType)]
#[alohomora_out_type(to_derive = [Serialize])]
//#[derive(Serialize, Default)]
pub struct ApplicationRow {
    ////#[serde(rename = "Ev. č. přihlášky")]
    pub application: BBox<i32, AnyPolicy>,
    //#[serde(rename = "Jméno")]
    pub name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Příjmení")]
    pub surname: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Rodné příjmení (pokud odlišné)")]
    pub birth_surname: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Místo narození")]
    pub birthplace: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Datum narození")]
    pub birthdate: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Adresa trvalého pobytu")]
    pub address: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Adresa pro doručování písemností (pokud odlišné)")]
    pub letter_address: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Telefon")]
    pub telephone: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Státní občanství")]
    pub citizenship: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Email")]
    pub email: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Pohlaví")]
    pub sex: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Rodné číslo")]
    pub personal_identification_number: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Název školy (IZO)")]
    pub school_name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Zdravotní pojištění")]
    pub health_insurance: Option<BBox<String, AnyPolicy>>,

    //#[serde(rename = "Vysvědčení 1/8")]
    pub diploma_1_8: BBox<String, AnyPolicy>,
    //#[serde(rename = "Vysvědčení 2/8")]
    pub diploma_2_8: BBox<String, AnyPolicy>,
    //#[serde(rename = "Vysvědčení 1/9")]
    pub diploma_1_9: BBox<String, AnyPolicy>,
    //#[serde(rename = "Vysvědčení 2/9")]
    pub diploma_2_9: BBox<String, AnyPolicy>,

    //#[serde(rename = "První škola - název")]
    pub first_school_name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "První škola - obor")]
    pub first_school_field: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Druhá škola - název")]
    pub second_school_name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Druhá škola - obor")]
    pub second_school_field: Option<BBox<String, AnyPolicy>>,

    //#[serde(rename = "Jméno zákonného zástupce")]
    pub parent_name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Příjmení zákonného zástupce")]
    pub parent_surname: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Telefon zákonného zástupce")]
    pub parent_telephone: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Email zákonného zástupce")]
    pub parent_email: Option<BBox<String, AnyPolicy>>,

    //#[serde(rename = "Jméno druhého zákonného zástupce")]
    pub second_parent_name: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Příjmení druhého zákonného zástupce")]
    pub second_parent_surname: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Telefon druhého zákonného zástupce")]
    pub second_parent_telephone: Option<BBox<String, AnyPolicy>>,
    //#[serde(rename = "Email druhého zákonného zástupce")]
    pub second_parent_email: Option<BBox<String, AnyPolicy>>,
}

impl From<ApplicationRowOut> for portfolio_sandbox::ApplicationRow {
    fn from(value: ApplicationRowOut) -> Self {
        portfolio_sandbox::ApplicationRow { application: value.application, name: value.name, surname: value.surname, birth_surname: value.birth_surname, birthplace: value.birthplace, birthdate: value.birthdate, address: value.address, letter_address: value.letter_address, telephone: value.telephone, citizenship: value.citizenship, email: value.email, sex: value.sex, personal_identification_number: value.personal_identification_number, school_name: value.school_name, health_insurance: value.health_insurance, diploma_1_8: value.diploma_1_8, diploma_2_8: value.diploma_2_8, diploma_1_9: value.diploma_1_9, diploma_2_9: value.diploma_2_9, first_school_name: value.first_school_name, first_school_field: value.first_school_field, second_school_name: value.second_school_name, second_school_field: value.second_school_field, parent_name: value.parent_name, parent_surname: value.parent_surname, parent_telephone: value.parent_telephone, parent_email: value.parent_email, second_parent_name: value.second_parent_name, second_parent_surname: value.second_parent_surname, second_parent_telephone: value.second_parent_telephone, second_parent_email: value.second_parent_email }
    }
}

impl Default for ApplicationRow {
    fn default() -> Self {
        ApplicationRow {
            application: BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default())),
            ..Default::default()
        }
    }
}