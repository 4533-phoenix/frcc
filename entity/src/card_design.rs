//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.8

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "card_design")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub team: i32,
    pub name: String,
    pub note: String,
    pub year: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::card::Entity")]
    Card,
    #[sea_orm(
        belongs_to = "super::team::Entity",
        from = "Column::Team",
        to = "super::team::Column::Number",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Team,
}

impl Related<super::card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Card.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
