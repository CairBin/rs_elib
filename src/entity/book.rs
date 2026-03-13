use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "books")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub cover_path: Option<String>,
    pub isbn: Option<String>,
    pub category: Option<String>,
    pub created_by: Option<i32>,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::book_group::Entity")]
    BookGroups,
}

impl Related<super::group::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_group::Relation::Group.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::book_group::Relation::Book.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
