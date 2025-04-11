//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.8

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "team")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub number: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::card_design::Entity")]
    CardDesign,
}

impl Related<super::card_design::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CardDesign.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
