use alohomora::{bbox::BBox, policy::NoPolicy};
use chrono::NaiveDateTime;
use entity::{application, candidate};
use sea_orm::{EntityTrait, DbErr, DbConn, ModelTrait, FromQueryResult, QuerySelect, JoinType, RelationTrait, QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait};
use crate::Query;

const PAGE_SIZE: u64 = 20;

#[derive(FromQueryResult, Clone)]
pub struct ApplicationCandidateJoin {
    pub application_id: BBox<i32, NoPolicy>,
    pub personal_id_number: BBox<String, NoPolicy>,
    pub candidate_id: BBox<i32, NoPolicy>,
    pub name: Option<BBox<String, NoPolicy>>,
    pub surname: Option<BBox<String, NoPolicy>>,
    pub email: Option<BBox<String, NoPolicy>>,
    pub telephone: Option<BBox<String, NoPolicy>>,
    pub field_of_study: Option<BBox<String, NoPolicy>>,
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
        application_id: BBox<i32, NoPolicy>,
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
        field_of_study: Option<BBox<String, NoPolicy>>,
        // TODO(babman): highly likely that page and sort do not need to be BBoxes.
        page: Option<BBox<u64, NoPolicy>>,
        sort: Option<BBox<String, NoPolicy>>,
    ) -> Result<Vec<ApplicationCandidateJoin>, DbErr> {
        let select = application::Entity::find();

        // Are we sorting?
        let (column, order) = match sort {
            None => (application::Column::Id, sea_orm::Order::Asc),
            Some(sort) => get_ordering(sort.discard_box()),
        };

        // Are we filtering out by field_of_study?
        let select = match field_of_study {
            None => select,
            Some(field_of_study) => select.filter(application::Column::FieldOfStudy.eq(field_of_study)),
        };

        // Rest of the query.
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

        // Do we have pagination?
        match page {
            None => query.all(db).await,
            Some(page) => query
                .paginate(db, PAGE_SIZE)
                .fetch_page(page.discard_box())
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
        candidate_id: BBox<i32, NoPolicy>,
    ) -> Result<Vec<application::Model>, DbErr> {
        let applications = application::Entity::find()
            .filter(application::Column::CandidateId.eq(candidate_id))
            .all(db)
            .await?;

        Ok(applications)
    }
}