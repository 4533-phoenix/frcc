//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.8

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    pub password: String,
    pub invited_with_code: Option<String>,
    pub is_admin: bool,
    pub is_verified: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::auth_token::Entity")]
    AuthToken,
    #[sea_orm(has_many = "super::invite::Entity")]
    Invite,
    #[sea_orm(has_many = "super::scan::Entity")]
    Scan,
    #[sea_orm(has_one = "super::user_team::Entity")]
    UserTeam,
}

impl Related<super::auth_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}

impl Related<super::invite::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invite.def()
    }
}

impl Related<super::scan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scan.def()
    }
}

impl Related<super::user_team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserTeam.def()
    }
}

impl Related<super::card::Entity> for Entity {
    fn to() -> RelationDef {
        super::scan::Relation::Card.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::scan::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
