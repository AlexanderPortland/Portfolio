use alohomora::{bbox::BBox, policy::NoPolicy};
use entity::{parent, candidate};
use sea_orm::DbConn;

use crate::{error::ServiceError, Mutation, models::{candidate_details::{EncryptedParentDetails}, candidate::ParentDetails}, Query};

pub struct ParentService;

impl ParentService {
    pub async fn create(
        db: &DbConn,
        application_id: BBox<i32, alohomora::policy::NoPolicy>,
    ) -> Result<parent::Model, ServiceError> {
        let parent = Mutation::create_parent(db, application_id)
            .await?;

        Ok(parent)
    }

    pub async fn add_parents_details(
        db: &DbConn,
        ref_candidate: &candidate::Model,
        parents_details: &Vec<ParentDetails>,
        recipients: &BBox<Vec<String>, NoPolicy>, // put the bbox on the outside bc whole list will have 
                                                  // same policy
    ) -> Result<Vec<parent::Model>, ServiceError> {
        if parents_details.len() > 2 {
            return Err(ServiceError::ParentOverflow);
        }
        
        let found_parents = Query::find_candidate_parents(db, ref_candidate).await?;

        let mut result = vec![];
        for i in 0..parents_details.len() {
            let found_parent = match found_parents.get(i) {
                Some(parent) => parent.to_owned(),
                None => ParentService::create(db, ref_candidate.id.clone()).await?,
            };
            let enc_details = EncryptedParentDetails::new(&parents_details[i], &recipients.clone().discard_box()).await?;
            let parent = Mutation::add_parent_details(db, found_parent, enc_details.clone()).await?;
            result.push(parent);
        }

        // delete parents that are not in the form
        for i in parents_details.len()..found_parents.len() {
            Mutation::delete_parent(db, found_parents[i].to_owned()).await?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use alohomora::{bbox::BBox, policy::NoPolicy};
    use once_cell::sync::Lazy;

    use crate::{utils::db::get_memory_sqlite_connection, models::{candidate::{ParentDetails, ApplicationDetails, CandidateDetails}, candidate_details::EncryptedApplicationDetails, grade::GradeList, school::School}, services::{candidate_service::{CandidateService, tests::put_user_data}, application_service::ApplicationService, parent_service::ParentService}, crypto};

    pub static APPLICATION_DETAILS_TWO_PARENTS: Lazy<Mutex<ApplicationDetails>> = Lazy::new(|| 
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
                email: BBox::new("parent_email".to_string(), NoPolicy::new()),
            },
            ParentDetails {
                name: BBox::new("parent_name2".to_string(), NoPolicy::new()),
                surname: BBox::new("parent_surname2".to_string(), NoPolicy::new()),
                telephone: BBox::new("parent_telephone2".to_string(), NoPolicy::new()),
                email: BBox::new("parent_email2".to_string(), NoPolicy::new()),
            }],
        })
    );

    #[tokio::test]
    async fn create_parent_test() {
        let db = get_memory_sqlite_connection().await;
        let candidate = CandidateService::create(&db, BBox::new("".to_string(), NoPolicy::new())).await.unwrap();
        super::ParentService::create(&db, candidate.id.clone()).await.unwrap();
        super::ParentService::create(&db, candidate.id).await.unwrap();
    }

    #[tokio::test]
    async fn add_parent_details_test() {
        let db = get_memory_sqlite_connection().await;
        let plain_text_password = "test".to_string();
        // let application = ApplicationService::create(&"".to_string(), NoPolicy::new()), &db, 103100, &plain_text_password, "".to_string(), NoPolicy::new())).await.unwrap();
        let (application, candidate, _) = put_user_data(&db).await;

        ParentService::create(&db, candidate.id.clone()).await.unwrap();

        let form = APPLICATION_DETAILS_TWO_PARENTS.lock().unwrap().clone();

        let (candidate, parents) = ApplicationService::add_all_details(&db, &application, candidate.clone(), &form)
            .await
            .unwrap();

        let priv_key = BBox::new(
            crypto::decrypt_password(application.private_key.clone().discard_box(), plain_text_password).await.unwrap(), NoPolicy::new());
        let dec_details = EncryptedApplicationDetails::try_from((&candidate, &parents))
            .unwrap()
            .decrypt(priv_key)
            .await
            .unwrap();

        assert_eq!(dec_details.candidate.name, form.candidate.name);
        assert_eq!(dec_details.candidate.surname, form.candidate.surname);
        assert_eq!(dec_details.candidate.birthplace, form.candidate.birthplace);
        assert_eq!(dec_details.candidate.birthdate, form.candidate.birthdate);
        assert_eq!(dec_details.candidate.address, form.candidate.address);
        assert_eq!(dec_details.candidate.telephone, form.candidate.telephone);
        assert_eq!(dec_details.candidate.citizenship, form.candidate.citizenship);
        assert_eq!(dec_details.candidate.email, form.candidate.email);
        assert_eq!(dec_details.candidate.sex, form.candidate.sex);
        assert_eq!(dec_details.candidate.personal_id_number, BBox::new("0000001111".to_string(), NoPolicy::new()));
        assert_eq!(dec_details.candidate.school_name, form.candidate.school_name);
        assert_eq!(dec_details.candidate.health_insurance, form.candidate.health_insurance);
        assert_eq!(dec_details.candidate.grades, form.candidate.grades);
        assert_eq!(dec_details.candidate.test_language, form.candidate.test_language);

        assert_eq!(dec_details.parents.len(), form.parents.len());
        for i in 0..dec_details.parents.len() {
            assert_eq!(dec_details.parents[i].name, form.parents[i].name);
            assert_eq!(dec_details.parents[i].surname, form.parents[i].surname);
            assert_eq!(dec_details.parents[i].telephone, form.parents[i].telephone);
            assert_eq!(dec_details.parents[i].email, form.parents[i].email);
        }
    }
}