use alohomora::{bbox::BBox, policy::NoPolicy};
use chrono::NaiveDate;

use entity::{candidate, parent};
use futures::future;

use crate::{crypto, models::candidate::ApplicationDetails, error::ServiceError, utils::date::parse_naive_date_from_opt_str};

use super::{candidate::{CandidateDetails, ParentDetails}, grade::GradeList, school::School};

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Clone)]
pub struct EncryptedString(String);

#[derive(Debug, Clone)]
pub struct EncryptedCandidateDetails {
    pub name: Option<BBox<EncryptedString, NoPolicy>>,
    pub surname: Option<BBox<EncryptedString, NoPolicy>>,
    pub birth_surname: Option<BBox<EncryptedString, NoPolicy>>,
    pub birthplace: Option<BBox<EncryptedString, NoPolicy>>,
    pub birthdate: Option<BBox<EncryptedString, NoPolicy>>,
    pub address: Option<BBox<EncryptedString, NoPolicy>>,
    pub letter_address: Option<BBox<EncryptedString, NoPolicy>>,
    pub telephone: Option<BBox<EncryptedString, NoPolicy>>,
    pub citizenship: Option<BBox<EncryptedString, NoPolicy>>,
    pub email: Option<BBox<EncryptedString, NoPolicy>>,
    pub sex: Option<BBox<EncryptedString, NoPolicy>>,
    pub personal_id_number: Option<BBox<EncryptedString, NoPolicy>>,
    pub school_name: Option<BBox<EncryptedString, NoPolicy>>,
    pub health_insurance: Option<BBox<EncryptedString, NoPolicy>>,
    pub grades_json: Option<BBox<EncryptedString, NoPolicy>>,
    pub first_school: Option<BBox<EncryptedString, NoPolicy>>,
    pub second_school: Option<BBox<EncryptedString, NoPolicy>>,
    pub test_language: Option<BBox<String, NoPolicy>>,
}

#[derive(Debug, Clone)]
pub struct EncryptedParentDetails {
    pub name: Option<BBox<EncryptedString, NoPolicy>>,
    pub surname: Option<BBox<EncryptedString, NoPolicy>>,
    pub telephone: Option<BBox<EncryptedString, NoPolicy>>,
    pub email: Option<BBox<EncryptedString, NoPolicy>>,
}
#[derive(Debug, Clone)]
pub struct EncryptedApplicationDetails {
    pub candidate: EncryptedCandidateDetails,
    pub parents: Vec<EncryptedParentDetails>,
}

impl EncryptedString {
    pub async fn new(s: &str, recipients: &Vec<BBox<String, NoPolicy>>) -> Result<Self, ServiceError> {
        let recipients = recipients.iter()
            .map(|s| s.as_ref().discard_box().as_str())
            .collect();
        let encrypted_string = crypto::encrypt_password_with_recipients(&s, &recipients).await?;
        Ok(Self(encrypted_string))
    }

    pub async fn new_option(s: &BBox<String, NoPolicy>, recipients: &Vec<String>) -> Result<Option<BBox<Self, NoPolicy>>, ServiceError> {
        match s.clone().discard_box().is_empty() {
            true => Ok(None),
            false => {
                let recipients = recipients.iter().map(|s| &**s).collect();
                let encrypted_s = crypto::encrypt_password_with_recipients(&s.clone().discard_box(), &recipients).await?;
                Ok(Some(BBox::new(Self(encrypted_s), NoPolicy::new())))
            },
        }
    }

    pub async fn decrypt(&self, private_key: &String) -> Result<String, ServiceError> {
        crypto::decrypt_password_with_private_key(&self.0, private_key).await
    }

    pub async fn decrypt_option(
        s: &Option<BBox<EncryptedString, NoPolicy>>,
        private_key: &BBox<String, NoPolicy>,
    ) -> Result<Option<BBox<String, NoPolicy>>, ServiceError> {
        match s.as_ref() {
            Some(s) => {
                let a = s.clone().discard_box().decrypt(&private_key.clone().discard_box()).await?;

                Ok(Some(BBox::new(a, NoPolicy::new())))
            },
            None => Ok(None),
        }
    }

    pub fn to_string(self) -> String {
        self.0
    }
}

impl Into<String> for EncryptedString {
    fn into(self) -> String {
        self.0
    }
}

impl TryFrom<&Option<String>> for EncryptedString {
    type Error = ServiceError;

    fn try_from(s: &Option<String>) -> Result<Self, Self::Error> {
        match s {
            Some(s) => Ok(Self(s.to_owned())),
            None => Err(ServiceError::CandidateDetailsNotSet),
        }
    }
}

impl From<String> for EncryptedString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl TryFrom<Option<NaiveDate>> for EncryptedString {
    type Error = ServiceError;

    fn try_from(d: Option<NaiveDate>) -> Result<Self, Self::Error> {
        match d {
            Some(d) => Ok(Self(d.to_string())),
            None => Err(ServiceError::CandidateDetailsNotSet),
        }
    }
}

impl EncryptedCandidateDetails {
    pub async fn new(
        form: &CandidateDetails,
        recipients: &Vec<String>,
    ) -> Result<EncryptedCandidateDetails, ServiceError> {
        let birthdate_str = BBox::new(form.birthdate.clone().discard_box().format(NAIVE_DATE_FMT).to_string(),
            NoPolicy::new());
        let grades_str = BBox::new(form.grades.clone().discard_box().to_string(), NoPolicy::new());
        let (first_school_str, second_school_str) = 
            (BBox::new(form.first_school.clone().discard_box().to_string(), NoPolicy::new()), 
            BBox::new(form.second_school.clone().discard_box().to_string(), NoPolicy::new()));
        let d = tokio::try_join!(
            EncryptedString::new_option(&form.name, recipients),
            EncryptedString::new_option(&form.surname, recipients),
            EncryptedString::new_option(&form.birth_surname, recipients),
            EncryptedString::new_option(&form.birthplace, recipients),
            EncryptedString::new_option(&birthdate_str, recipients),
            EncryptedString::new_option(&form.address, recipients),
            EncryptedString::new_option(&form.letter_address, recipients),
            EncryptedString::new_option(&form.telephone, recipients),
            EncryptedString::new_option(&form.citizenship, recipients),
            EncryptedString::new_option(&form.email, recipients),
            EncryptedString::new_option(&form.sex, recipients),
            EncryptedString::new_option(&form.personal_id_number, recipients),
            EncryptedString::new_option(&form.school_name, recipients),
            EncryptedString::new_option(&form.health_insurance, recipients),
            EncryptedString::new_option(&grades_str, recipients),
            EncryptedString::new_option(&first_school_str, recipients),
            EncryptedString::new_option(&second_school_str, recipients),
        )?;

        Ok(
            EncryptedCandidateDetails {
                name: d.0,
                surname: d.1,
                birth_surname: d.2,
                birthplace: d.3,
                birthdate: d.4,
                address: d.5,
                letter_address: d.6,
                telephone: d.7,
                citizenship: d.8,
                email: d.9,
                sex: d.10,
                personal_id_number: d.11,
                school_name: d.12,
                health_insurance: d.13,
                grades_json: d.14,
                first_school: d.15,
                second_school: d.16,
                test_language: Some(form.test_language.clone()),
            }
        )
    }

    pub async fn decrypt(&self, priv_key: &BBox<String, NoPolicy>) -> Result<CandidateDetails, ServiceError> {
        let d = tokio::try_join!(
            EncryptedString::decrypt_option(&self.name, priv_key),              // 0
            EncryptedString::decrypt_option(&self.surname, priv_key),           // 1
            EncryptedString::decrypt_option(&self.birth_surname, priv_key),     // 2
            EncryptedString::decrypt_option(&self.birthplace, priv_key),        // 3
            EncryptedString::decrypt_option(&self.birthdate, priv_key),         // 4
            EncryptedString::decrypt_option(&self.address, priv_key),           // 5
            EncryptedString::decrypt_option(&self.letter_address, priv_key),    // 6
            EncryptedString::decrypt_option(&self.telephone, priv_key),         // 7
            EncryptedString::decrypt_option(&self.citizenship, priv_key),       // 8
            EncryptedString::decrypt_option(&self.email, priv_key),             // 9
            EncryptedString::decrypt_option(&self.sex, priv_key),               // 10
            EncryptedString::decrypt_option(&self.personal_id_number, priv_key),// 11
            EncryptedString::decrypt_option(&self.school_name, priv_key),       // 12
            EncryptedString::decrypt_option(&self.health_insurance, priv_key),  // 13
            EncryptedString::decrypt_option(&self.grades_json, priv_key),       // 14
            EncryptedString::decrypt_option(&self.first_school, priv_key),      // 15
            EncryptedString::decrypt_option(&self.second_school, priv_key),     // 16
        )?;

        Ok(CandidateDetails {
                name: d.0.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                surname: d.1.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                birth_surname: d.2.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                birthplace: d.3.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                birthdate: parse_naive_date_from_opt_str(d.4, NAIVE_DATE_FMT)?,
                address: d.5.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                letter_address: d.6.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                telephone: d.7.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                citizenship: d.8.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                email: d.9.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                sex: d.10.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                personal_id_number: d.11.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                school_name: d.12.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                health_insurance: d.13.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                grades: BBox::new(GradeList::from_opt_str(d.14).unwrap_or(GradeList::from(vec![])), NoPolicy::new()),
                first_school: BBox::new(School::from_opt_str(d.15).unwrap_or_default(), NoPolicy::new()),
                second_school: BBox::new(School::from_opt_str(d.16).unwrap_or_default(), NoPolicy::new()),
                test_language: self.test_language.clone().unwrap_or(BBox::new(String::from(""), NoPolicy::new())),
            }
        )
    }

    pub fn is_filled(&self) -> bool {
        self.name.is_some() &&
        self.surname.is_some() &&
        self.birthplace.is_some() &&
        self.birthdate.is_some() &&
        self.address.is_some() &&
        self.telephone.is_some() &&
        self.citizenship.is_some() &&
        self.email.is_some() &&
        self.personal_id_number.is_some() &&
        self.school_name.is_some() &&
        self.health_insurance.is_some() &&
        self.first_school.is_some() &&
        self.second_school.is_some()

    }
}
impl From<&candidate::Model> for EncryptedCandidateDetails {
    fn from(
        candidate: &candidate::Model,
    ) -> Self {
        let a = candidate.name.clone();
        EncryptedCandidateDetails {
            name: try_encrypt_bbox_str(&candidate.name),
            surname: try_encrypt_bbox_str(&candidate.surname),
            birth_surname: try_encrypt_bbox_str(&candidate.birth_surname),
            birthplace: try_encrypt_bbox_str(&candidate.birthplace),
            birthdate: try_encrypt_bbox_str(&candidate.birthdate),
            address: try_encrypt_bbox_str(&candidate.address),
            letter_address: try_encrypt_bbox_str(&candidate.letter_address),
            telephone: try_encrypt_bbox_str(&candidate.telephone),
            citizenship: try_encrypt_bbox_str(&candidate.citizenship),
            email: try_encrypt_bbox_str(&candidate.email),
            sex: try_encrypt_bbox_str(&candidate.sex),
            personal_id_number: encrypt_bbox_str(candidate.personal_identification_number.to_owned()),
            school_name: try_encrypt_bbox_str(&candidate.school_name),
            health_insurance: try_encrypt_bbox_str(&candidate.health_insurance),
            grades_json: try_encrypt_bbox_str(&candidate.grades_json),
            first_school: try_encrypt_bbox_str(&candidate.first_school),
            second_school: try_encrypt_bbox_str(&candidate.second_school),
            test_language: candidate.test_language.to_owned(),
        }
    }
}

pub fn try_encrypt_bbox_str(b: &Option<BBox<String, NoPolicy>>) -> Option<BBox<EncryptedString, NoPolicy>> {
    match b.as_ref() {
        None => None,
        Some(s) => {
            let o = EncryptedString::try_from(s.clone().discard_box()).ok()?;
            Some(BBox::new(o, NoPolicy::new()))
        },
    }
}

pub fn encrypt_bbox_str(b: BBox<String, NoPolicy>) -> Option<BBox<EncryptedString, NoPolicy>> {
    match b.discard_box() {
        s => {
            let s = EncryptedString::from(s);
            Some(BBox::new(s, NoPolicy::new()))
        },
    }
}

//fn encrypted_string_from_

impl EncryptedParentDetails {
    pub async fn new(
        form: &ParentDetails,
        recipients: &Vec<String>,
    ) -> Result<EncryptedParentDetails, ServiceError> {
        let d = tokio::try_join!(
            EncryptedString::new_option(&form.name, recipients),
            EncryptedString::new_option(&form.surname, recipients),
            EncryptedString::new_option(&form.telephone, recipients),
            EncryptedString::new_option(&form.email, recipients),
        )?;

        Ok(
            EncryptedParentDetails {
                name: d.0,
                surname: d.1,
                telephone: d.2,
                email: d.3,
            }
        )
    }

    pub async fn decrypt(&self, priv_key: &BBox<String, NoPolicy>) -> Result<ParentDetails, ServiceError> {
        let d = tokio::try_join!(
            EncryptedString::decrypt_option(&self.name, &priv_key),
            EncryptedString::decrypt_option(&self.surname, &priv_key),
            EncryptedString::decrypt_option(&self.telephone, &priv_key),
            EncryptedString::decrypt_option(&self.email, &priv_key),
        )?;

        Ok(ParentDetails {
                name: d.0.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                surname: d.1.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                telephone: d.2.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
                email: d.3.unwrap_or(BBox::new("".to_string(), NoPolicy::new())),
            }
        )
    }

    pub fn is_filled(&self) -> bool {
        self.name.is_some() &&
        self.surname.is_some() &&
        self.telephone.is_some() &&
        self.email.is_some()
    }
}
impl From<&parent::Model> for EncryptedParentDetails {
    fn from(
        parent: &parent::Model,
    ) -> Self {
        EncryptedParentDetails { 
            name: try_encrypt_bbox_str(&parent.name),
            surname: try_encrypt_bbox_str(&parent.surname),
            telephone: try_encrypt_bbox_str(&parent.telephone),
            email: try_encrypt_bbox_str(&parent.email),
        }
    }
}

impl EncryptedApplicationDetails {
    pub async fn new(
        form: &ApplicationDetails,
        recipients: &Vec<String>,
    ) -> Result<EncryptedApplicationDetails, ServiceError> {
        let candidate =  EncryptedCandidateDetails::new(&form.candidate, &recipients).await?;
        let enc_parents = future::try_join_all(
            form.parents.iter()
                .map(|d| EncryptedParentDetails::new(d, &recipients))
        ).await?;
        Ok(
            EncryptedApplicationDetails {
                candidate,
                parents: enc_parents,
            }
        )
    }

    pub async fn decrypt(self, priv_key: BBox<String, NoPolicy>) -> Result<ApplicationDetails, ServiceError> {
        let decrypted_candidate = self.candidate.decrypt(&priv_key).await?;

        let decrypted_parents = future::try_join_all(
            self.parents
                .iter()
                .map(|d| d.decrypt(&priv_key))
        ).await?;

        Ok(ApplicationDetails {
            candidate: decrypted_candidate,
            parents: decrypted_parents,
        })
    }

    pub fn is_filled(&self) -> bool {
        self.candidate.is_filled() &&
        self.parents.iter().all(|p| p.is_filled())
    }
}

impl From<(&candidate::Model, &Vec<parent::Model>)> for EncryptedApplicationDetails {
    fn from(
        (candidate, parents): (&candidate::Model, &Vec<parent::Model>),
    ) -> Self {
        let enc_parents = parents.iter()
            .map(|m| EncryptedParentDetails::from(m))
            .collect::<Vec<EncryptedParentDetails>>();

        EncryptedApplicationDetails {
            candidate: EncryptedCandidateDetails::from(candidate),
            parents: enc_parents,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Mutex;

    use alohomora::{bbox::BBox, policy::NoPolicy};
    use chrono::Local;
    use entity::admin;
    use once_cell::sync::Lazy;
    use sea_orm::{DbConn, Set, ActiveModelTrait};

    use crate::{crypto, models::{candidate::{CandidateDetails, ParentDetails}, grade::GradeList, school::School}, utils::db::get_memory_sqlite_connection, services::candidate_service::tests::put_user_data};

    use super::{ApplicationDetails, EncryptedApplicationDetails, EncryptedString};

    const PUBLIC_KEY: &str = "age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5";
    const PRIVATE_KEY: &str = "AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS";

    pub static APPLICATION_DETAILS: Lazy<Mutex<ApplicationDetails>> = Lazy::new(|| 
        Mutex::new(ApplicationDetails {
            candidate: CandidateDetails {
                name: BBox::new("name".to_string(), NoPolicy::new()),
                surname: BBox::new("surname".to_string(), NoPolicy::new()),
                birth_surname: BBox::new("birth_surname".to_string(), NoPolicy::new()),
                birthplace: BBox::new("birthplace".to_string(), NoPolicy::new()),
                birthdate: BBox::new(chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(), NoPolicy::new()),
                address: BBox::new("address".to_string(), NoPolicy::new()),
                letter_address: BBox::new("letter_address".to_string(), NoPolicy::new()),
                telephone: BBox::new("telephone".to_string(), NoPolicy::new()),
                citizenship: BBox::new("citizenship".to_string(), NoPolicy::new()),
                email: BBox::new("email".to_string(), NoPolicy::new()),
                sex: BBox::new("sex".to_string(), NoPolicy::new()),
                personal_id_number: BBox::new("personal_id_number".to_string(), NoPolicy::new()),
                school_name: BBox::new("school_name".to_string(), NoPolicy::new()),
                health_insurance: BBox::new("health_insurance".to_string(), NoPolicy::new()),
                grades: BBox::new(GradeList::from(vec![]), NoPolicy::new()),
                first_school: BBox::new(School::from_opt_str(Some(BBox::new("{\"name\": \"SSPS\", \"field\": \"KB\"}".to_string(), NoPolicy::new()))).unwrap(), NoPolicy::new()),
                second_school: BBox::new(School::from_opt_str(Some(BBox::new("{\"name\": \"SSPS\", \"field\": \"IT\"}".to_string(), NoPolicy::new()))).unwrap(), NoPolicy::new()),
                test_language: BBox::new("test_language".to_string(), NoPolicy::new()),
            },
            parents: vec![ParentDetails {
                name: BBox::new("parent_name".to_string(), NoPolicy::new()),
                surname: BBox::new("parent_surname".to_string(), NoPolicy::new()),
                telephone: BBox::new("parent_telephone".to_string(), NoPolicy::new()),
                email: BBox::new("parent_email".to_string(), NoPolicy::new())
            }]
        })
    );

    pub fn assert_all_application_details(details: &ApplicationDetails) {
        assert_eq!(details.candidate.name, BBox::new("name".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.surname, BBox::new("surname".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.birthplace, BBox::new("birthplace".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.birthdate, BBox::new(chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(), NoPolicy::new()));
        assert_eq!(details.candidate.address, BBox::new("address".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.telephone, BBox::new("telephone".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.citizenship, BBox::new("citizenship".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.email, BBox::new("email".to_string(), NoPolicy::new()));
        assert_eq!(details.candidate.sex, BBox::new("sex".to_string(), NoPolicy::new()));
        for parent in &details.parents {
            assert_eq!(parent.name, BBox::new("parent_name".to_string(), NoPolicy::new()));
            assert_eq!(parent.surname, BBox::new("parent_surname".to_string(), NoPolicy::new()));
            assert_eq!(parent.telephone, BBox::new("parent_telephone".to_string(), NoPolicy::new()));
            assert_eq!(parent.email, BBox::new("parent_email".to_string(), NoPolicy::new()));
        }
    }

    async fn insert_test_admin(db: &DbConn) -> admin::Model {
        admin::ActiveModel {
            id: Set(BBox::new(1, NoPolicy::new())),
            name: Set(BBox::new("Admin".to_owned(), NoPolicy::new())),
            public_key: Set(BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_owned(), NoPolicy::new())),
            // AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS
            private_key: Set(BBox::new("5KCEGk0ueWVGnu5Xo3rmpLoilcVZ2ZWmwIcdZEJ8rrBNW7jwzZU/XTcTXtk/xyy/zjF8s+YnuVpOklQvX3EC/Sn+ZwyPY3jokM2RNwnZZlnqdehOEV1SMm/Y".to_owned(), NoPolicy::new())),
            // test
            password: Set(BBox::new("$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ".to_owned(), NoPolicy::new())),
            created_at: Set(BBox::new(Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_encrypted_application_details_new() {
        let encrypted_details = EncryptedApplicationDetails::new(
            &APPLICATION_DETAILS.lock().unwrap().clone(),
            &vec![PUBLIC_KEY.to_string()],
        )
        .await
        .unwrap();

        assert_eq!(
            crypto::decrypt_password_with_private_key(&encrypted_details.candidate.name.unwrap().discard_box().0, PRIVATE_KEY)
                .await
                .unwrap(),
            "name"
        );
        assert_eq!(
            crypto::decrypt_password_with_private_key(&encrypted_details.candidate.email.unwrap().discard_box().0, PRIVATE_KEY)
                .await
                .unwrap(),
            "email"
        );
        assert_eq!(
            crypto::decrypt_password_with_private_key(&encrypted_details.candidate.sex.unwrap().discard_box().0, PRIVATE_KEY)
                .await
                .unwrap(),
            "sex"
        );
    }

    #[tokio::test]
    async fn test_encrypted_application_details_decrypt() {
        let encrypted_details = EncryptedApplicationDetails::new(
            &APPLICATION_DETAILS.lock().unwrap().clone(),
            &vec![PUBLIC_KEY.to_string()],
        )
        .await
        .unwrap();

        let application_details = encrypted_details
            .decrypt(BBox::new(PRIVATE_KEY.to_string(), NoPolicy::new()))
            .await
            .unwrap();

        assert_all_application_details(&application_details);
    }

    #[tokio::test]
    async fn test_encrypted_application_details_from_candidate_parent() {
        let db = get_memory_sqlite_connection().await;
        let _admin = insert_test_admin(&db).await;

        let (_, candidate, parents) = put_user_data(&db).await;

        let encrypted_details = EncryptedApplicationDetails::try_from((&candidate, &parents)).unwrap();

        let application_details = encrypted_details
            .decrypt(BBox::new(PRIVATE_KEY.to_string(), NoPolicy::new())) // decrypt with admin's private key
            .await
            .unwrap();

        assert_all_application_details(&application_details);
    }

    #[tokio::test]
    async fn test_encrypted_string_new() {
        let encrypted = EncryptedString::new(
            "test",
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy {})]
        ).await.unwrap();

        assert_eq!(
            crypto::decrypt_password_with_private_key(&encrypted.0, PRIVATE_KEY)
                .await
                .unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_encrypted_string_decrypt() {
        let encrypted = EncryptedString::new(
            "test",
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy {})]
        ).await.unwrap();

        assert_eq!(
            encrypted.decrypt(&PRIVATE_KEY.to_string()).await.unwrap(),
            "test"
        );
    }
}
