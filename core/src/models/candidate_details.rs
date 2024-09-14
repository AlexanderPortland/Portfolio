use std::any::{type_name, Any};
use std::fmt::Debug;
use alohomora::bbox::BBox;
use alohomora::orm::ORMPolicy;
use alohomora::policy::{AnyPolicy, Policy, NoPolicy};
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use chrono::NaiveDate;
use alohomora::sandbox::execute_sandbox;

use entity::{candidate, parent};
use futures::future;
use portfolio_policies::key::KeyPolicy;
use portfolio_policies::FakePolicy;
use portfolio_sandbox::naive_date_str;
use serde::Serialize;

use crate::{crypto, models::candidate::ApplicationDetails, error::ServiceError, utils::date::parse_naive_date_from_opt_str};
use crate::crypto_helpers::{my_decrypt_password_with_private_key, my_encrypt_password_with_recipients};

use super::grade::Grade;
use super::{candidate::{CandidateDetails, ParentDetails}, grade::GradeList, school::School};

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

#[derive(Debug, Clone)]
pub struct EncryptedString(BBox<String, AnyPolicy>);

#[derive(Debug, Clone)]
pub struct EncryptedCandidateDetails {
    pub name: Option<EncryptedString>,
    pub surname: Option<EncryptedString>,
    pub birth_surname: Option<EncryptedString>,
    pub birthplace: Option<EncryptedString>,
    pub birthdate: Option<EncryptedString>,
    pub address: Option<EncryptedString>,
    pub letter_address: Option<EncryptedString>,
    pub telephone: Option<EncryptedString>,
    pub citizenship: Option<EncryptedString>,
    pub email: Option<EncryptedString>,
    pub sex: Option<EncryptedString>,
    pub personal_id_number: Option<EncryptedString>,
    pub school_name: Option<EncryptedString>,
    pub health_insurance: Option<EncryptedString>,
    pub grades_json: Option<EncryptedString>,
    pub first_school: Option<EncryptedString>,
    pub second_school: Option<EncryptedString>,
    pub test_language: Option<BBox<String, AnyPolicy>>,
}

#[derive(Debug, Clone)]
pub struct EncryptedParentDetails {
    pub name: Option<EncryptedString>,
    pub surname: Option<EncryptedString>,
    pub telephone: Option<EncryptedString>,
    pub email: Option<EncryptedString>,
}
#[derive(Debug, Clone)]
pub struct EncryptedApplicationDetails {
    pub candidate: EncryptedCandidateDetails,
    pub parents: Vec<EncryptedParentDetails>,
}

impl EncryptedString {
    pub async fn new<P1: Policy + Clone + 'static>(
        password_plain_text: BBox<String, P1>,
        recipients: &Vec<BBox<String, NoPolicy>>,
    ) -> Result<Self, ServiceError> {
        let encrypted_string = my_encrypt_password_with_recipients(password_plain_text, recipients).await?;
        Ok(Self(encrypted_string.into_any_policy()))
    }

    pub async fn new_option<P1: Policy + Clone + 'static>(
        password_plain_text: &BBox<String, P1>,
        recipients: &Vec<BBox<String, NoPolicy>>,
    ) -> Result<Option<Self>, ServiceError> {
        let password_plain_text = password_plain_text.clone().into_ppr(
            PrivacyPureRegion::new(|password: String|
                if password.is_empty() {
                    None
                } else {
                    Some(password)
                }
            )
        );

        match password_plain_text.transpose() {
            None => Ok(None),
            Some(password_plain_text) => {
                let encrypted_string = my_encrypt_password_with_recipients(password_plain_text, recipients).await?;
                Ok(Some(Self(encrypted_string.into_any_policy())))
            }
        }
    }

    pub async fn decrypt<P1: Policy + Clone + 'static>(
        self,
        private_key: &BBox<String, P1>
    ) -> Result<BBox<String, AnyPolicy>, ServiceError> {
        my_decrypt_password_with_private_key(self.0, private_key.clone()).await
    }

    pub async fn decrypt_option<P1: Policy + Clone + 'static>(
        self_: &Option<Self>,
        private_key: &BBox<String, P1>,
    ) -> Result<Option<BBox<String, AnyPolicy>>, ServiceError> {
        match self_ {
            None => Ok(None),
            Some(self_) => Ok(Some(self_.clone().decrypt(private_key).await?)),
        }
    }
}

impl<P: Policy + Clone + 'static> TryFrom<&Option<BBox<String, P>>> for EncryptedString {
    type Error = ServiceError;

    fn try_from(s: &Option<BBox<String, P>>) -> Result<Self, Self::Error> {
        match s {
            Some(s) => Ok(Self(s.clone().into_any_policy())),
            None => Err(ServiceError::CandidateDetailsNotSet),
        }
    }
}

impl<P: Policy + Clone + 'static>  From<BBox<String, P>> for EncryptedString {
    fn from(s: BBox<String, P>) -> Self {
        Self(s.into_any_policy())
    }
}

impl<P: Policy + Clone + 'static> Into<BBox<String, P>> for EncryptedString {
    fn into(self) -> BBox<String, P> {
        self.0.specialize_policy().unwrap()
    }
}

impl<P: Policy + Clone + 'static> TryFrom<Option<BBox<NaiveDate, P>>> for EncryptedString {
    type Error = ServiceError;

    fn try_from(d: Option<BBox<NaiveDate, P>>) -> Result<Self, Self::Error> {
        match d {
            None => Err(ServiceError::CandidateDetailsNotSet),
            Some(d) => Ok(Self(
                naive_date_str_caller(d.into_any_policy(), false)
            )),
        }
    }
}

pub fn naive_date_str_caller(date: BBox<NaiveDate, AnyPolicy>, format: bool) -> BBox<String, AnyPolicy> {
    // date.into_ppr(PrivacyPureRegion::new(|date: NaiveDate|{
    //     naive_date_str((date, format))
    // }))
    execute_sandbox::<naive_date_str, _, _>((date, format))
}

// FIXME: this will go in SANDBOX 
// (DRAFTED)
// fn naive_date_str((date, format): (chrono::NaiveDate, bool)) -> String {
//     match format {
//         true => date.to_string(),
//         false => date.format(NAIVE_DATE_FMT).to_string(),
//     }
// }

fn serde_grade_sandbox_caller(t: BBox<GradeList, AnyPolicy>) -> BBox<String, AnyPolicy> {
    let s: BBox<portfolio_sandbox::GradeList, AnyPolicy> = t.into_ppr(PrivacyPureRegion::new(|s: GradeList|{
        s.to_sandbox()
    }));
    execute_sandbox::<portfolio_sandbox::serde_from_grade, _, _>(s)

    // execute_sandbox::<portfolio_sandbox::serde_from_grade, _, _>(t)
}

// FIXME: this will go in SANDBOX lib
// drafted
// fn serde_grade_sandbox(t: GradeList) -> String {
//     serde_json::to_string(&t).unwrap()
// }

fn serde_school_sandbox_caller(t: BBox<School, AnyPolicy>) -> BBox<String, AnyPolicy> {
    // t.into_ppr(PrivacyPureRegion::new(|t|{
    //     serde_school_sandbox(t)
    // }))
    let s: BBox<portfolio_sandbox::School, AnyPolicy> = t.into_ppr(PrivacyPureRegion::new(|s: School|{
        s.to_sandbox()
    }));
    execute_sandbox::<portfolio_sandbox::serde_from_school, _, _>(s)
}

// FIXME: this will go in SANDBOX lib
// drafted
// fn serde_school_sandbox(t: School) -> String {
//     serde_json::to_string(&t).unwrap()
// }

impl EncryptedCandidateDetails {
    pub async fn new(
        form: &CandidateDetails,
        recipients: &Vec<BBox<String, NoPolicy>>,
    ) -> Result<EncryptedCandidateDetails, ServiceError> {
        let birthdate_str = naive_date_str_caller(form.birthdate.clone(), true);
        let grades_str = serde_grade_sandbox_caller(form.grades.clone());
        let first_school_str = serde_school_sandbox_caller(form.firstSchool.clone());
        let second_school_str = serde_school_sandbox_caller(form.secondSchool.clone());
        let d = tokio::try_join!(
            EncryptedString::new_option(&form.name, recipients),
            EncryptedString::new_option(&form.surname, recipients),
            EncryptedString::new_option(&form.birthSurname, recipients),
            EncryptedString::new_option(&form.birthplace, recipients),
            EncryptedString::new_option(&birthdate_str, recipients),
            EncryptedString::new_option(&form.address, recipients),
            EncryptedString::new_option(&form.letterAddress, recipients),
            EncryptedString::new_option(&form.telephone, recipients),
            EncryptedString::new_option(&form.citizenship, recipients),
            EncryptedString::new_option(&form.email, recipients),
            EncryptedString::new_option(&form.sex, recipients),
            EncryptedString::new_option(&form.personalIdNumber, recipients),
            EncryptedString::new_option(&form.schoolName, recipients),
            EncryptedString::new_option(&form.healthInsurance, recipients),
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
                test_language: Some(form.testLanguage.clone()),
            }
        )
    }

    pub async fn decrypt(&self, private_key: &BBox<String, KeyPolicy>) -> Result<CandidateDetails, ServiceError> {
        let d = tokio::try_join!(
            EncryptedString::decrypt_option(&self.name, private_key),              // 0
            EncryptedString::decrypt_option(&self.surname, private_key),           // 1
            EncryptedString::decrypt_option(&self.birth_surname, private_key),     // 2
            EncryptedString::decrypt_option(&self.birthplace, private_key),        // 3
            EncryptedString::decrypt_option(&self.birthdate, private_key),         // 4
            EncryptedString::decrypt_option(&self.address, private_key),           // 5
            EncryptedString::decrypt_option(&self.letter_address, private_key),    // 6
            EncryptedString::decrypt_option(&self.telephone, private_key),         // 7
            EncryptedString::decrypt_option(&self.citizenship, private_key),       // 8
            EncryptedString::decrypt_option(&self.email, private_key),             // 9
            EncryptedString::decrypt_option(&self.sex, private_key),               // 10
            EncryptedString::decrypt_option(&self.personal_id_number, private_key),// 11
            EncryptedString::decrypt_option(&self.school_name, private_key),       // 12
            EncryptedString::decrypt_option(&self.health_insurance, private_key),  // 13
            EncryptedString::decrypt_option(&self.grades_json, private_key),       // 14
            EncryptedString::decrypt_option(&self.first_school, private_key),      // 15
            EncryptedString::decrypt_option(&self.second_school, private_key),     // 16
        )?;

        Ok(CandidateDetails {
                name: d.0.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                surname: d.1.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                birthSurname: d.2.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                birthplace: d.3.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                birthdate: parse_naive_date_from_opt_str(d.4, NAIVE_DATE_FMT)?,
                address: d.5.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                letterAddress: d.6.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                telephone: d.7.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                citizenship: d.8.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                email: d.9.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                sex: d.10.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                personalIdNumber: d.11.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                schoolName: d.12.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                healthInsurance: d.13.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                grades: GradeList::from_opt_str(d.14).unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                firstSchool: School::from_opt_str(d.15).unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                secondSchool: School::from_opt_str(d.16).unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                testLanguage: self.test_language.clone().unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
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
        EncryptedCandidateDetails {
            name: EncryptedString::try_from(&candidate.name).ok(),
            surname: EncryptedString::try_from(&candidate.surname).ok(),
            birth_surname: EncryptedString::try_from(&candidate.birth_surname).ok(),
            birthplace: EncryptedString::try_from(&candidate.birthplace).ok(),
            birthdate: EncryptedString::try_from(&candidate.birthdate).ok(),
            address: EncryptedString::try_from(&candidate.address).ok(),
            letter_address: EncryptedString::try_from(&candidate.letter_address).ok(),
            telephone: EncryptedString::try_from(&candidate.telephone).ok(),
            citizenship: EncryptedString::try_from(&candidate.citizenship).ok(),
            email: EncryptedString::try_from(&candidate.email).ok(),
            sex: EncryptedString::try_from(&candidate.sex).ok(),
            personal_id_number: Some(EncryptedString::from(candidate.personal_identification_number.to_owned())),
            school_name: EncryptedString::try_from(&candidate.school_name).ok(),
            health_insurance: EncryptedString::try_from(&candidate.health_insurance).ok(),
            grades_json: EncryptedString::try_from(&candidate.grades_json).ok(),
            first_school: EncryptedString::try_from(&candidate.first_school).ok(),
            second_school: EncryptedString::try_from(&candidate.second_school).ok(),
            test_language: candidate.test_language.to_owned().map(|b| b.into_any_policy()),
        }
    }
}

//fn encrypted_string_from_

impl EncryptedParentDetails {
    pub async fn new(
        form: &ParentDetails,
        recipients: &Vec<BBox<String, NoPolicy>>,
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

    pub async fn decrypt<P: Policy + Clone + 'static>(&self, private_key: &BBox<String, P>) -> Result<ParentDetails, ServiceError> {
        let d = tokio::try_join!(
            EncryptedString::decrypt_option(&self.name, &private_key),
            EncryptedString::decrypt_option(&self.surname, &private_key),
            EncryptedString::decrypt_option(&self.telephone, &private_key),
            EncryptedString::decrypt_option(&self.email, &private_key),
        )?;

        Ok(ParentDetails {
                name: d.0.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                surname: d.1.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                telephone: d.2.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
                email: d.3.unwrap_or(BBox::new(Default::default(), AnyPolicy::new(NoPolicy::default()))),
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
            name: EncryptedString::try_from(&parent.name).ok(),
            surname: EncryptedString::try_from(&parent.surname).ok(),
            telephone: EncryptedString::try_from(&parent.telephone).ok(),
            email: EncryptedString::try_from(&parent.email).ok(),
        }
    }
}

impl EncryptedApplicationDetails {
    pub async fn new(
        form: &ApplicationDetails,
        recipients: &Vec<BBox<String, NoPolicy>>,
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

    pub async fn decrypt(self, private_key: BBox<String, KeyPolicy>) -> Result<ApplicationDetails, ServiceError> {
        let decrypted_candidate = self.candidate.decrypt(&private_key).await?;

        let decrypted_parents = future::try_join_all(
            self.parents
                .iter()
                .map(|d| {
                    d.decrypt(&private_key)
                    // match d.decrypt(&private_key).await {
                    //     Ok(o) => {Ok(o)},
                    //     Err(e) => {Err(e)},
                    // }
                    
                })
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

    use portfolio_policies::{key::KeyPolicy, FakePolicy};
    use portfolio_policies::data::CandidateDataPolicy;
    use alohomora::{bbox::BBox, pcr::{execute_pcr, PrivacyCriticalRegion, Signature}, policy::{AnyPolicy, NoPolicy}, pure::PrivacyPureRegion};
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
                name: BBox::new("name".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                surname: BBox::new("surname".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                birthSurname: BBox::new("birth_surname".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                birthplace: BBox::new("birthplace".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                birthdate: BBox::new(chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                address: BBox::new("address".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                letterAddress: BBox::new("letter_address".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                telephone: BBox::new("telephone".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                citizenship: BBox::new("citizenship".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                email: BBox::new("email".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                sex: BBox::new("sex".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                personalIdNumber: BBox::new("personal_id_number".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                schoolName: BBox::new("school_name".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                healthInsurance: BBox::new("health_insurance".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                grades: BBox::new(GradeList::from(vec![]), AnyPolicy::new(CandidateDataPolicy::new(None))),
                firstSchool: School::from_opt_str(Some(BBox::new("{\"name\": \"SSPS\", \"field\": \"KB\"}".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))))).unwrap(),
                secondSchool: School::from_opt_str(Some(BBox::new("{\"name\": \"SSPS\", \"field\": \"IT\"}".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))))).unwrap(),
                testLanguage: BBox::new("test_language".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
            },
            parents: vec![ParentDetails {
                name: BBox::new("parent_name".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                surname: BBox::new("parent_surname".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                telephone: BBox::new("parent_telephone".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))),
                email: BBox::new("parent_email".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None)))
            }]
        })
    );

    pub fn assert_all_application_details(details: &ApplicationDetails) {
        assert_eq!(details.candidate.name, BBox::new("name".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.surname, BBox::new("surname".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.birthplace, BBox::new("birthplace".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.birthdate, BBox::new(chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.address, BBox::new("address".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.telephone, BBox::new("telephone".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.citizenship, BBox::new("citizenship".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.email, BBox::new("email".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        assert_eq!(details.candidate.sex, BBox::new("sex".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        for parent in &details.parents {
            assert_eq!(parent.name, BBox::new("parent_name".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
            assert_eq!(parent.surname, BBox::new("parent_surname".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
            assert_eq!(parent.telephone, BBox::new("parent_telephone".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
            assert_eq!(parent.email, BBox::new("parent_email".to_string(), AnyPolicy::new(CandidateDataPolicy::new(None))));
        }
    }

    async fn insert_test_admin(db: &DbConn) -> admin::Model {
        admin::ActiveModel {
            id: Set(BBox::new(1, FakePolicy::new())),
            name: Set(BBox::new("Admin".to_owned(), FakePolicy::new())),
            public_key: Set(BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_owned(), NoPolicy::new())),
            // AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS
            private_key: Set(BBox::new("5KCEGk0ueWVGnu5Xo3rmpLoilcVZ2ZWmwIcdZEJ8rrBNW7jwzZU/XTcTXtk/xyy/zjF8s+YnuVpOklQvX3EC/Sn+ZwyPY3jokM2RNwnZZlnqdehOEV1SMm/Y".to_owned(), KeyPolicy::new(None, portfolio_policies::key::KeySource::JustGenerated))),
            // test
            password: Set(BBox::new("$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ".to_owned(), FakePolicy::new())),
            created_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
            updated_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
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
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy::new())],
        )
        .await
        .unwrap();

        // let dec_key_eq = PrivacyCriticalRegion::new(|enc_password: String, _, _|{
        //     crypto::decrypt_password_with_private_key(&enc_password, PRIVATE_KEY)
        // });

        let (name, email, sex) = execute_pcr((
                encrypted_details.candidate.name.unwrap().0, 
                encrypted_details.candidate.email.unwrap().0, 
                encrypted_details.candidate.sex.unwrap().0), 
            PrivacyCriticalRegion::new(|(name, email, sex), _, _|{
                (name, email, sex)
            },
            Signature{username: "AlexanderPortland", signature: ""}, 
            Signature{username: "AlexanderPortland", signature: ""}, 
            Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();

        assert_eq!(crypto::decrypt_password_with_private_key(&name, PRIVATE_KEY).await.unwrap(), "name");
        assert_eq!(crypto::decrypt_password_with_private_key(&email, PRIVATE_KEY).await.unwrap(), "email");
        assert_eq!(crypto::decrypt_password_with_private_key(&sex, PRIVATE_KEY).await.unwrap(), "sex");
    }

    #[tokio::test]
    async fn test_encrypted_application_details_decrypt() {
        let encrypted_details = EncryptedApplicationDetails::new(
            &APPLICATION_DETAILS.lock().unwrap().clone(),
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy::new())],
        )
        .await
        .unwrap();

        let application_details = encrypted_details
            .decrypt(BBox::new(PRIVATE_KEY.to_string(), KeyPolicy::new(None, portfolio_policies::key::KeySource::JustGenerated)))
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
            .decrypt(BBox::new(PRIVATE_KEY.to_string(), KeyPolicy::new(None, portfolio_policies::key::KeySource::JustGenerated))) // decrypt with admin's private key
            .await
            .unwrap();

        assert_all_application_details(&application_details);
    }

    #[tokio::test]
    async fn test_encrypted_string_new() {
        let encrypted = EncryptedString::new(
            BBox::new("test".to_string(), FakePolicy::new()),
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy {})]
        ).await.unwrap();

        let enc_password = execute_pcr(encrypted.0, 
            PrivacyCriticalRegion::new(|enc_password: String, _, _|{
            enc_password
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();

        assert_eq!(
            crypto::decrypt_password_with_private_key(&enc_password, PRIVATE_KEY).await.unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_encrypted_string_decrypt() {
        let encrypted = EncryptedString::new(
            BBox::new("test".to_string(), FakePolicy::new()),
            &vec![BBox::new(PUBLIC_KEY.to_string(), NoPolicy {})]
        ).await.unwrap();

        assert_eq!(
            encrypted.decrypt(&BBox::new(PRIVATE_KEY.to_string(), FakePolicy::new())).await.unwrap().specialize_policy().unwrap(),
            BBox::new("test".to_string(), FakePolicy::new())
        );
    }
}
