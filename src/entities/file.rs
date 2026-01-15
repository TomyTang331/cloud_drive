use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,

    /// File owner ID
    pub user_id: i32,

    /// File/folder name
    pub name: String,

    /// Relative path (relative to user root)
    pub path: String,

    /// Parent directory path
    pub parent_path: String,

    /// File type: file or folder
    pub file_type: String,

    /// MIME type (files only)
    #[sea_orm(nullable)]
    pub mime_type: Option<String>,

    /// File size in bytes (files only)
    #[sea_orm(nullable)]
    pub size_bytes: Option<i64>,

    /// Physical storage path
    pub storage_path: String,

    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
