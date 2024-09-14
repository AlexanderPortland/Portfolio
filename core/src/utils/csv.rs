use std::any::Any;

use alohomora::{bbox::BBox, fold::fold, policy::AnyPolicy, pure::{execute_pure, PrivacyPureRegion}, sandbox::{self, execute_sandbox}, AlohomoraType};
use portfolio_sandbox::serde_from_tuple;
use crate::{
    error::ServiceError, models::{application::{ApplicationRow, ApplicationRowOut}, candidate::{ApplicationDetails, CandidateRowOut}, candidate_details::EncryptedApplicationDetails}, services::application_service::ApplicationService, Query
};
use alohomora::{context::Context, pcr::PrivacyCriticalRegion, policy::NoPolicy};
use sea_orm::DbConn;
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Serialize;
use portfolio_policies::{key::KeyPolicy, FakePolicy};
use crate::models::candidate::{CandidateRow, FieldOfStudy, FieldsCombination};
use crate::models::candidate_details::EncryptedCandidateDetails;
use crate::models::grade::GradeList;
use crate::models::school::School;

impl TryFrom<(BBox<i32, FakePolicy>, ApplicationDetails)> for ApplicationRow {
    type Error = ServiceError;
    fn try_from((application, d): (BBox<i32, FakePolicy>, ApplicationDetails)) -> Result<Self, ServiceError> {
        let c = d.candidate;

        type Tup = (GradeList, GradeList, GradeList, GradeList);
        let diplomas = c.grades.clone().into_ppr(PrivacyPureRegion::new(|grades: GradeList| {
            grades.group_by_semester()
        })).transpose()?;

        pub fn serde_from_tuple_caller(d: BBox<Tup, AnyPolicy>, i: u8) -> BBox<String, AnyPolicy> {
            let t = d.into_ppr(PrivacyPureRegion::new(|d: (GradeList, GradeList, GradeList, GradeList)|{
                (d.0.to_sandbox(), d.1.to_sandbox(), d.2.to_sandbox(), d.3.to_sandbox())
            }));
            execute_sandbox::<serde_from_tuple, _, _>((t, i))
        }

        // FIXME: figure out some sandbox folding here
        let diploma_1_8 = serde_from_tuple_caller(diplomas.clone(), 0);
        let diploma_2_8 = serde_from_tuple_caller(diplomas.clone(), 1);
        let diploma_1_9 = serde_from_tuple_caller(diplomas.clone(), 2);
        let diploma_2_9 = serde_from_tuple_caller(diplomas, 3);


        let first_school_name = c.firstSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.name().to_string()));
        let first_school_field = c.firstSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.field().to_string()));
        let second_school_name = c.secondSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.name().to_string()));
        let second_school_field = c.secondSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.field().to_string()));

        Ok(Self {
            application: application.into_any_policy(),
            name: Some(c.name),
            surname: Some(c.surname),
            birth_surname: Some(c.birthSurname),
            birthplace: Some(c.birthplace),
            birthdate: Some(crate::models::candidate_details::naive_date_str_caller(c.birthdate, false)),
            address: Some(c.address),
            letter_address: Some(c.letterAddress),
            telephone: Some(c.telephone),
            citizenship: Some(c.citizenship),
            email: Some(c.email),
            sex: Some(c.sex),
            personal_identification_number: Some(c.personalIdNumber),
            health_insurance: Some(c.healthInsurance),
            school_name: Some(c.schoolName),

            diploma_1_8,
            diploma_2_8,
            diploma_1_9,
            diploma_2_9,

            first_school_name: Some(first_school_name),
            first_school_field: Some(first_school_field),
            second_school_name: Some(second_school_name),
            second_school_field: Some(second_school_field),

            parent_name: d.parents.get(0).map(|p| p.name.clone()),
            parent_surname: d.parents.get(0).map(|p| p.surname.clone()),
            parent_telephone: d.parents.get(0).map(|p| p.telephone.clone()),
            parent_email: d.parents.get(0).map(|p| p.email.clone()),

            second_parent_name: d.parents.get(1).map(|p| p.name.clone()),
            second_parent_surname: d.parents.get(1).map(|p| p.surname.clone()),
            second_parent_telephone: d.parents.get(1).map(|p| p.telephone.clone()),
            second_parent_email: d.parents.get(1).map(|p| p.email.clone()),
        })
    }
}

pub fn error_map(err: portfolio_sandbox::ServiceError) -> ServiceError { err.into() }

pub fn serialize_cand_row_caller(rows: Vec<CandidateRow>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> {
    let sandbox_rows = rows.into_iter().map(|row|{
        execute_pure(row, PrivacyPureRegion::new(|row: CandidateRowOut|{
            row.into()
        })).unwrap()
    }).collect::<Vec<BBox<portfolio_sandbox::CandidateRow, AnyPolicy>>>();

    let b: Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> = execute_pure(sandbox_rows, PrivacyPureRegion::new(|rows|{
        portfolio_sandbox::serialize_cand_row(rows).map_err(error_map)
    })).unwrap().transpose();
    b
}

pub fn serialize_app_row_caller(rows: Vec<ApplicationRow>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> {
    let sandbox_rows = rows.into_iter().map(|row|{
        execute_pure(row, PrivacyPureRegion::new(|row: ApplicationRowOut|{
            row.into()
        })).unwrap()
    }).collect::<Vec<BBox<portfolio_sandbox::ApplicationRow, AnyPolicy>>>();

    let b: Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> = execute_pure(sandbox_rows, PrivacyPureRegion::new(|rows|{
        portfolio_sandbox::serialize_app_row(rows).map_err(error_map)
    })).unwrap().transpose();
    b
}

// This should be a Sandboxed region.
pub fn serialize_in_sandbox<T: AlohomoraType>(rows: Vec<T>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> where T::Out: Serialize {
    execute_pure(rows, PrivacyPureRegion::new(|rows| {
        let mut wtr = csv::Writer::from_writer(vec![]);
        for row in rows {
            wtr.serialize(row).unwrap();
        }
        wtr.into_inner().map_err(|_| ServiceError::CsvIntoInnerError)
    })).unwrap().transpose()
}

#[async_trait]
pub trait CsvExporter {
    async fn export(db: &DbConn, private_key: BBox<String, KeyPolicy>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError>;
}

pub struct ApplicationCsv;

#[async_trait]
impl CsvExporter for ApplicationCsv {
    async fn export(db: &DbConn, private_key: BBox<String, KeyPolicy>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> {
        println!("Exportin");
        let applications = Query::list_applications_compact(&db).await?;
        println!("listed apps");
        let mut rows = Vec::new();
        for application in applications {
            println!("pre-cand");
            let candidate = ApplicationService::find_related_candidate(db, &application).await?;
            println!("pre-par");
            let parents = Query::find_candidate_parents(db, &candidate).await?;
            println!("pre-row");
            let row: ApplicationRow = match EncryptedApplicationDetails::try_from((&candidate, &parents))
            {
                Ok(d) => {
                    println!("yay");
                    ApplicationRow::try_from(
                    d.decrypt(private_key.clone())
                        .await
                        .map(|d| (application.id.clone(), d))?,
                )
                    .unwrap_or(ApplicationRow {
                        application: application.id.into_any_policy(),
                        ..Default::default()
                    })},

                Err(_) => {
                    println!("nay");
                    ApplicationRow {
                    application: application.id.into_any_policy(),
                    ..Default::default()
                }},
            };
            println!("pre-push");
            rows.push(row);
        }
        println!("serialize?");
        serialize_in_sandbox(rows)
    }
}

pub struct CandidateCsv;

#[async_trait]
impl CsvExporter for CandidateCsv {
    async fn export(db: &DbConn, private_key: BBox<String, KeyPolicy>) -> Result<BBox<Vec<u8>, AnyPolicy>, ServiceError> {
        let candidates = Query::list_candidates_full(&db).await?;
        let applications = Query::list_applications_compact(&db).await?;
        let parents = Query::list_all_parents(&db).await?;

        let mut rows = Vec::new();
        for model in candidates {
            let (id, c) = (
                model.id.clone(),
                EncryptedCandidateDetails::from(&model).decrypt(&private_key).await?
            );

            let related_applications = applications
                .iter()
                .filter(|a| a.candidate_id.clone() == id)
                .map(|a| a.id.clone())
                .collect::<Vec<_>>();

            let parents = parents
                .iter()
                .filter(|p| p.candidate_id.clone() == id)
                .cloned()
                .collect::<Vec<_>>();

            let first_field = c.firstSchool.clone().into_ppr(
                PrivacyPureRegion::new(|f: School|
                    get_our_school_field(&f)
                )
            ).transpose().map_err(|_| ServiceError::InvalidFieldOfStudy)?;

            let second_field = c.secondSchool.clone().into_ppr(
                PrivacyPureRegion::new(|f: School|
                    get_our_school_field(&f)
                )
            ).transpose().map_err(|_| ServiceError::InvalidFieldOfStudy)?;

            let applications_fields_comb = get_applications_fields_comb(&related_applications);
            let fields_combination = execute_pure(
                (first_field.clone(), second_field.clone()),
                PrivacyPureRegion::new(|(first_field, second_field)| {
                    FieldsCombination::from_fields(&first_field, &second_field)
                }),
            ).unwrap().specialize_policy::<FakePolicy>().unwrap();

            let fields_match = execute_pure(
                (applications_fields_comb, fields_combination.clone()),
                PrivacyPureRegion::new(|(applications_fields_comb, fields_combination)| {
                    fields_combination == applications_fields_comb
                }),
            ).unwrap().specialize_policy::<FakePolicy>().unwrap();

            let first_school_name = c.firstSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.name().to_string()));
            let first_school_field = c.firstSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.field().to_string()));
            let second_school_name = c.secondSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.name().to_string()));
            let second_school_field = c.secondSchool.clone().into_ppr(PrivacyPureRegion::new(|s: School| s.field().to_string()));

            let row = CandidateRow {
                id: id.into_any_policy(),
                first_application: related_applications.first().ok_or(ServiceError::CandidateNotFound)?.clone().into_any_policy(),
                second_application: related_applications.get(1).map(|id| id.clone().into_any_policy()),
                first_school: first_school_name,
                first_school_field,
                second_school: second_school_name,
                second_school_field,
                first_day_admissions: first_field.clone().into_ppr(PrivacyPureRegion::new(|f: Option<FieldOfStudy>| f.is_some())),
                second_day_admissions: second_field.clone().into_ppr(PrivacyPureRegion::new(|f: Option<FieldOfStudy>| f.is_some())),
                first_day_field: first_field.transpose(),
                second_day_field: second_field.transpose(),
                fields_combination: fields_combination.into_any_policy(),
                personal_id_number: c.personalIdNumber,
                fields_match: fields_match.into_any_policy(),
                name: c.name.to_owned(),
                surname: c.surname.to_owned(),
                email: c.email.to_owned(),
                telephone: c.telephone.to_owned(),
                parent_email: parents.first().map(|parent| parent.email.clone().map(|b| b.into_any_policy())).unwrap_or(None),
                parent_telephone: parents.first().map(|parent| parent.telephone.clone().map(|b| b.into_any_policy())).unwrap_or(None),
            };
            rows.push(row);
        }


        // This should be a Sandboxed region.
        serialize_in_sandbox(rows)
    }
}

fn get_applications_fields_comb(
    related_applications: &[BBox<i32, FakePolicy>],
) -> BBox<FieldsCombination, FakePolicy> {
    let fields_vec = related_applications
        .iter()
        .map(|id| {
            id.clone().into_ppr(PrivacyPureRegion::new(|id| {
                FieldOfStudy::from(id)
            }))
        })
        .collect::<Vec<_>>();

    let fields_vec: BBox<Vec<_>, _> = fold(fields_vec)
        .unwrap()
        .specialize_policy()
        .unwrap();
    fields_vec.into_ppr(PrivacyPureRegion::new(|fields_vec: Vec<FieldOfStudy>| {
        FieldsCombination::from_fields(
            &fields_vec.get(0).map(|f| f.clone()),
            &fields_vec.get(1).map(|f| f.clone()),
        )
    }))
}

fn get_our_school_field(school: &School) -> Result<Option<FieldOfStudy>, ServiceError> {
    if school.name() == "Smíchovská střední průmyslová škola a gymnázium" {
        Ok(
            Some(
                FieldOfStudy::try_from(school.field().to_owned())?
            )
        )
    } else {
        Ok(None)
    }
}

