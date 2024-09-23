use alohomora::{bbox::BBox, policy::NoPolicy};
use chrono::NaiveDateTime;
use entity::{application, candidate};
use sea_orm::{EntityTrait, DbErr, DbConn, ModelTrait, FromQueryResult, QuerySelect, JoinType, RelationTrait, QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait};
use portfolio_policies::{data::CandidateDataPolicy, FakePolicy};
use crate::Query;

const PAGE_SIZE: u64 = 20;

#[derive(FromQueryResult, Clone)]
pub struct ApplicationCandidateJoin {
    pub application_id: BBox<i32, CandidateDataPolicy>,
    pub personal_id_number: BBox<String, CandidateDataPolicy>,
    pub candidate_id: BBox<i32, CandidateDataPolicy>,
    pub name: Option<BBox<String, CandidateDataPolicy>>,
    pub surname: Option<BBox<String, CandidateDataPolicy>>,
    pub email: Option<BBox<String, CandidateDataPolicy>>,
    pub telephone: Option<BBox<String, CandidateDataPolicy>>,
    pub field_of_study: Option<BBox<String, CandidateDataPolicy>>,
    pub created_at: BBox<NaiveDateTime, NoPolicy>,
}

fn get_ordering(sort: String) -> (application::Column, sea_orm::Order)
{
    let mut split = sort.split("_");
    let column = split.next();
    let order = split.next();

    let column = match column {
        Some("id") => application::Column::Id,
        Some("createdAt") => application::Column::CreatedAt,
        _ => application::Column::Id
    };

    let order = match order {
        Some("asc") => sea_orm::Order::Asc,
        Some("desc") => sea_orm::Order::Desc,
        _ => sea_orm::Order::Asc,
    };

    (column, order)
}

impl Query {
    pub async fn find_application_by_id(
        db: &DbConn,
        application_id: BBox<i32, CandidateDataPolicy>,
    ) -> Result<Option<application::Model>, DbErr> {
        application::Entity::find_by_id(application_id)
            .one(db)
            .await
    }

    pub async fn find_related_candidate(
        db: &DbConn,
        application: &application::Model,
    ) -> Result<Option<candidate::Model>, DbErr> {
        application
            .find_related(candidate::Entity)
            .one(db)
            .await
    }

    pub async fn list_applications(
        db: &DbConn,
        field_of_study: Option<String>,
        page: Option<u64>,
        sort: Option<String>,
    ) -> Result<Vec<ApplicationCandidateJoin>, DbErr> {
        // let timer = std::time::Instant::now();
        let select = application::Entity::find();
        // println!("{:?} for finding entity", timer.elapsed());

        // let timer = std::time::Instant::now();
        // Are we sorting?
        let (column, order) = match sort {
            None => (application::Column::Id, sea_orm::Order::Asc),
            Some(sort) => get_ordering(sort),
        };

        // Are we filtering out by field_of_study?
        let select = match field_of_study {
            None => select,
            Some(field_of_study) => select.filter(application::Column::FieldOfStudy.eq(field_of_study)),
        };
        // println!("{:?} sorted all that shit", timer.elapsed());

        // Rest of the query.
        // let timer = std::time::Instant::now();
        let query = select
            .order_by(column, order)
            .join(JoinType::InnerJoin, application::Relation::Candidate.def())
            .column_as(application::Column::Id, "application_id")
            .column_as(candidate::Column::Id, "candidate_id")
            .column_as(candidate::Column::Name, "name")
            .column_as(candidate::Column::Surname, "surname")
            .column_as(candidate::Column::Email, "email")
            .column_as(candidate::Column::Telephone, "telephone")
            .column_as(application::Column::CreatedAt, "created_at")
            .into_model::<ApplicationCandidateJoin>();
        // println!("{:?} query", timer.elapsed());

        // Do we have pagination?
        match page {
            None => query.all(db).await,
            Some(page) => query
                .paginate(db, PAGE_SIZE)
                .fetch_page(page)
                .await,
        }
    }

    pub async fn list_applications_compact(
        db: &DbConn,
    ) -> Result<Vec<application::Model>, DbErr> {
        application::Entity::find()
            .join(JoinType::InnerJoin, application::Relation::Candidate.def())
            .all(db)
            .await
    }

    pub async fn find_applications_by_candidate_id(
        db: &DbConn,
        candidate_id: BBox<i32, CandidateDataPolicy>,
    ) -> Result<Vec<application::Model>, DbErr> {
        // let timer = std::time::Instant::now();
        let applications = application::Entity::find()
            .filter(application::Column::CandidateId.eq(candidate_id))
            .all(db)
            .await?;
        // println!("{:?} found apps", timer.elapsed());
        Ok(applications)
    }
}