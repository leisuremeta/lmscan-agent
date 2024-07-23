use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "balance_tx")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub hash: String,
    #[sea_orm(primary_key)]
    pub address: String,
    pub free: BigDecimal,
    pub lock: BigDecimal,
    pub spend: bool,
    pub token: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn add_free(&self, a: BigDecimal) -> Model {
        Model {
            free: a + &self.free,
            ..self.clone()
        }
    }
    pub fn add_lock(&self, a: BigDecimal) -> Model {
        Model {
            lock: a + &self.lock,
            ..self.clone()
        }
    }
}
