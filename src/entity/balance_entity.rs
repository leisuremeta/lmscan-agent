use sea_orm::{entity::prelude::*, IntoActiveModel};

use crate::balance_entity;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "balance")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    pub free: BigDecimal,
    pub locked: BigDecimal,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn add(self, free: BigDecimal, lock: BigDecimal) -> Model {
        let next_f = &self.free + free;
        let next_l = &self.locked + lock;
        Model {
            free: next_f,
            locked: next_l,
            ..self
        }
    }
    pub fn to_bal(self) -> balance_entity::ActiveModel {
        balance_entity::Model {
            address: self.address,
            free: self.free,
            locked: self.locked,
            created_at: self.created_at,
            updated_at: self.updated_at,
        } .into_active_model()

    }
}
