use alohomora::{bbox::BBox, pcr::PrivacyCriticalRegion, policy::NoPolicy, AlohomoraType};
use alohomora_derive::ResponseBBoxJson;
use chrono::NaiveDateTime;
//use sea_orm::sea_query::private;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{database::query::application::ApplicationCandidateJoin, error::ServiceError};

use super::candidate_details::{try_encrypt_bbox_str, EncryptedString};

//#[derive(Debug, Serialize, Deserialize)]
#[derive(ResponseBBoxJson)]
////#[serde(rename_all = "camelCase")]
pub struct ApplicationResponse {
    pub application_id: BBox<i32, NoPolicy>,
    pub candidate_id: BBox<i32, NoPolicy>,
    pub related_applications: Vec<BBox<i32, NoPolicy>>,
    pub personal_id_number: BBox<String, NoPolicy>,
    pub name: BBox<String, NoPolicy>,
    pub surname: BBox<String, NoPolicy>,
    pub email: BBox<String, NoPolicy>,
    pub telephone: BBox<String, NoPolicy>,
    pub field_of_study: BBox<Option<String>, NoPolicy>,
    pub created_at: BBox<NaiveDateTime, NoPolicy>,
}

impl ApplicationResponse {
    pub async fn from_encrypted(
        private_key: &BBox<String, NoPolicy>,
        c: ApplicationCandidateJoin,
        related_applications: Vec<BBox<i32, NoPolicy>>,
    ) -> Result<Self, ServiceError> {
        let personal_id_number_str = EncryptedString::from(c.personal_id_number.discard_box().to_owned()).decrypt(&private_key.clone().discard_box()).await?;
        //let name = EncryptedString::decrypt_option(&EncryptedString::try_from(&c.name).ok(), private_key).await?;
        let name = EncryptedString::decrypt_option(&try_encrypt_bbox_str(&c.name), private_key).await?;
        let surname = EncryptedString::decrypt_option(&try_encrypt_bbox_str(&c.surname), private_key).await?;
        let email = EncryptedString::decrypt_option(&try_encrypt_bbox_str(&c.email), private_key).await?;
        let telephone = EncryptedString::decrypt_option(&try_encrypt_bbox_str(&c.telephone), private_key).await?;
        Ok(
            Self {
                application_id: c.application_id,
                candidate_id: c.candidate_id,
                related_applications,
                personal_id_number: BBox::new(personal_id_number_str, NoPolicy::new()),
                name: name.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                surname: surname.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                email: email.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                telephone:  telephone.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                field_of_study: c.field_of_study,
                created_at: c.created_at,
            }
        )
    }
}

// pub struct Tmp {
//     pub id: BBox<i32, NoPolicy>,
// }

// fn tmp_f(id: BBox<i32, NoPolicy>) {
//     let x = Tmp { id: id };
//     alohomora::unbox::unbox_(data, context, functor, arg)
//     let y = alohomora::fold::fold(x).unwrap();
//     y.unbox(BBox::new(None, NoPolicy::new()), PrivacyCriticalRegion::new(|y, _| {
//         let x = serde_json::to_string(y);
//     }), ());
// }

/// CSV export (admin endpoint)
#[derive(AlohomoraType)]
#[alohomora_out_type(to_derive = [Serialize])]
//#[derive(Serialize, Default)]
pub struct ApplicationRow {
    ////#[serde(rename = "Ev. č. přihlášky")]
    pub application: BBox<i32, NoPolicy>,
    //#[serde(rename = "Jméno")]
    pub name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Příjmení")]
    pub surname: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Rodné příjmení (pokud odlišné)")]
    pub birth_surname: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Místo narození")]
    pub birthplace: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Datum narození")]
    pub birthdate: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Adresa trvalého pobytu")]
    pub address: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Adresa pro doručování písemností (pokud odlišné)")]
    pub letter_address: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Telefon")]
    pub telephone: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Státní občanství")]
    pub citizenship: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Email")]
    pub email: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Pohlaví")]
    pub sex: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Rodné číslo")]
    pub personal_identification_number: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Název školy (IZO)")]
    pub school_name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Zdravotní pojištění")]
    pub health_insurance: BBox<Option<String>, NoPolicy>,

    //#[serde(rename = "Vysvědčení 1/8")]
    pub diploma_1_8: BBox<String, NoPolicy>,
    //#[serde(rename = "Vysvědčení 2/8")]
    pub diploma_2_8: BBox<String, NoPolicy>,
    //#[serde(rename = "Vysvědčení 1/9")]
    pub diploma_1_9: BBox<String, NoPolicy>,
    //#[serde(rename = "Vysvědčení 2/9")]
    pub diploma_2_9: BBox<String, NoPolicy>,

    //#[serde(rename = "První škola - název")]
    pub first_school_name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "První škola - obor")]
    pub first_school_field: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Druhá škola - název")]
    pub second_school_name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Druhá škola - obor")]
    pub second_school_field: BBox<Option<String>, NoPolicy>,

    //#[serde(rename = "Jméno zákonného zástupce")]
    pub parent_name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Příjmení zákonného zástupce")]
    pub parent_surname: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Telefon zákonného zástupce")]
    pub parent_telephone: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Email zákonného zástupce")]
    pub parent_email: BBox<Option<String>, NoPolicy>,

    //#[serde(rename = "Jméno druhého zákonného zástupce")]
    pub second_parent_name: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Příjmení druhého zákonného zástupce")]
    pub second_parent_surname: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Telefon druhého zákonného zástupce")]
    pub second_parent_telephone: BBox<Option<String>, NoPolicy>,
    //#[serde(rename = "Email druhého zákonného zástupce")]
    pub second_parent_email: BBox<Option<String>, NoPolicy>,
}

impl Default for ApplicationRow {
    fn default() -> Self {
        ApplicationRow{ application: BBox::new(0, NoPolicy::new()), 
            name: BBox::new(None, NoPolicy::new()), 
            surname: BBox::new(None, NoPolicy::new()), 
            birth_surname: BBox::new(None, NoPolicy::new()), 
            birthplace: BBox::new(None, NoPolicy::new()), 
            birthdate: BBox::new(None, NoPolicy::new()), 
            address: BBox::new(None, NoPolicy::new()), 
            letter_address: BBox::new(None, NoPolicy::new()), 
            telephone: BBox::new(None, NoPolicy::new()), 
            citizenship: BBox::new(None, NoPolicy::new()), 
            email: BBox::new(None, NoPolicy::new()), 
            sex: BBox::new(None, NoPolicy::new()), 
            personal_identification_number: BBox::new(None, NoPolicy::new()), 
            school_name: BBox::new(None, NoPolicy::new()), 
            health_insurance: BBox::new(None, NoPolicy::new()), 
            diploma_1_8: BBox::new("".to_string(), NoPolicy::new()), 
            diploma_2_8: BBox::new("".to_string(), NoPolicy::new()), 
            diploma_1_9: BBox::new("".to_string(), NoPolicy::new()), 
            diploma_2_9: BBox::new("".to_string(), NoPolicy::new()), 
            first_school_name: BBox::new(None, NoPolicy::new()), 
            first_school_field: BBox::new(None, NoPolicy::new()), 
            second_school_name: BBox::new(None, NoPolicy::new()), 
            second_school_field: BBox::new(None, NoPolicy::new()), 
            parent_name: BBox::new(None, NoPolicy::new()), 
            parent_surname: BBox::new(None, NoPolicy::new()), 
            parent_telephone: BBox::new(None, NoPolicy::new()), 
            parent_email: BBox::new(None, NoPolicy::new()), 
            second_parent_name: BBox::new(None, NoPolicy::new()), 
            second_parent_surname: BBox::new(None, NoPolicy::new()), 
            second_parent_telephone: BBox::new(None, NoPolicy::new()), 
            second_parent_email: BBox::new(None, NoPolicy::new()) }
    }
}