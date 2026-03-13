use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_group::Entity")]
    UserGroups,
    #[sea_orm(has_many = "super::book_group::Entity")]
    BookGroups,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_group::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_group::Relation::Group.def().rev())
    }
}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_group::Relation::Book.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::book_group::Relation::Group.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
