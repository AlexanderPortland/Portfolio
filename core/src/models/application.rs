use alohomora::{bbox::BBox, AlohomoraType};
use alohomora_derive::ResponseBBoxJson;
use chrono::NaiveDateTime;
//use sea_orm::sea_query::private;
use serde::Serialize;
use std::collections::HashMap;
use portfolio_policies::FakePolicy;

use crate::{database::query::application::ApplicationCandidateJoin, error::ServiceError};

use super::candidate_details::EncryptedString;

//#[derive(Debug, Serialize, Deserialize)]
#[derive(ResponseBBoxJson)]
pub struct ApplicationResponse {
    pub application_id: BBox<i32, FakePolicy>,
    pub candidate_id: BBox<i32, FakePolicy>,
    pub related_applications: Vec<BBox<i32, FakePolicy>>,
    pub personal_id_number: BBox<String, FakePolicy>,
    pub name: BBox<String, FakePolicy>,
    pub surname: BBox<String, FakePolicy>,
    pub email: BBox<String, FakePolicy>,
    pub telephone: BBox<String, FakePolicy>,
    pub field_of_study: Option<BBox<String, FakePolicy>>,
    pub created_at: BBox<NaiveDateTime, FakePolicy>,
}

impl ApplicationResponse {
    pub async fn from_encrypted(
        private_key: &BBox<String, FakePolicy>,
        c: ApplicationCandidateJoin,
        related_applications: Vec<BBox<i32, FakePolicy>>,
    ) -> Result<Self, ServiceError> {
        let personal_id_number_str = EncryptedString::from(c.personal_id_number.to_owned()).decrypt(private_key).await?;
        let name = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.name).ok(), private_key).await?;
        let surname = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.surname).ok(), private_key).await?;
        let email = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.email).ok(), private_key).await?;
        let telephone = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.telephone).ok(), private_key).await?;
        Ok(
            Self {
                application_id: c.application_id,
                candidate_id: c.candidate_id,
                related_applications,
                personal_id_number: personal_id_number_str,
                name: name.unwrap_or_default(),
                surname: surname.unwrap_or_default(),
                email: email.unwrap_or_default(),
                telephone:  telephone.unwrap_or_default(),
                field_of_study: c.field_of_study,
                created_at: c.created_at,
            }
        )
    }
}

/// CSV export (admin endpoint)
#[derive(AlohomoraType, Default)]
#[alohomora_out_type(to_derive = [Serialize])]
//#[derive(Serialize, Default)]
pub struct ApplicationRow {
    ////#[serde(rename = "Ev. č. přihlášky")]
    pub application: BBox<i32, FakePolicy>,
    //#[serde(rename = "Jméno")]
    pub name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Příjmení")]
    pub surname: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Rodné příjmení (pokud odlišné)")]
    pub birth_surname: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Místo narození")]
    pub birthplace: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Datum narození")]
    pub birthdate: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Adresa trvalého pobytu")]
    pub address: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Adresa pro doručování písemností (pokud odlišné)")]
    pub letter_address: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Telefon")]
    pub telephone: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Státní občanství")]
    pub citizenship: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Email")]
    pub email: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Pohlaví")]
    pub sex: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Rodné číslo")]
    pub personal_identification_number: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Název školy (IZO)")]
    pub school_name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Zdravotní pojištění")]
    pub health_insurance: Option<BBox<String, FakePolicy>>,

    //#[serde(rename = "Vysvědčení 1/8")]
    pub diploma_1_8: BBox<String, FakePolicy>,
    //#[serde(rename = "Vysvědčení 2/8")]
    pub diploma_2_8: BBox<String, FakePolicy>,
    //#[serde(rename = "Vysvědčení 1/9")]
    pub diploma_1_9: BBox<String, FakePolicy>,
    //#[serde(rename = "Vysvědčení 2/9")]
    pub diploma_2_9: BBox<String, FakePolicy>,

    //#[serde(rename = "První škola - název")]
    pub first_school_name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "První škola - obor")]
    pub first_school_field: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Druhá škola - název")]
    pub second_school_name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Druhá škola - obor")]
    pub second_school_field: Option<BBox<String, FakePolicy>>,

    //#[serde(rename = "Jméno zákonného zástupce")]
    pub parent_name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Příjmení zákonného zástupce")]
    pub parent_surname: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Telefon zákonného zástupce")]
    pub parent_telephone: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Email zákonného zástupce")]
    pub parent_email: Option<BBox<String, FakePolicy>>,

    //#[serde(rename = "Jméno druhého zákonného zástupce")]
    pub second_parent_name: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Příjmení druhého zákonného zástupce")]
    pub second_parent_surname: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Telefon druhého zákonného zástupce")]
    pub second_parent_telephone: Option<BBox<String, FakePolicy>>,
    //#[serde(rename = "Email druhého zákonného zástupce")]
    pub second_parent_email: Option<BBox<String, FakePolicy>>,
}