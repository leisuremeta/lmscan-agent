use bigdecimal::BigDecimal;

use crate::{library::common::now, balance_entity};

#[derive(Clone, Debug)]
pub struct Balance {
  free:   BigDecimal,
  locked: BigDecimal,
  upd_at: i64,
}

impl Balance {
  pub fn new(free: BigDecimal, locked: BigDecimal) -> Self {
    Self { free, locked, upd_at: now() }
  }

  pub fn from(entity: balance_entity::Model) -> Self {
    Self { free: entity.free, locked: entity.locked, upd_at: entity.updated_at }
  }

  pub fn default() -> Self {
    Self { free: BigDecimal::from(0), locked: BigDecimal::from(0), upd_at: now() }
  }

  pub fn add_free(&mut self, free_amount: &BigDecimal) {
    self.free += free_amount;
    self.upd_at = now();
  }

  pub fn sub_free(&mut self, free_amount: &BigDecimal) {
    self.free -= free_amount;
    self.upd_at = now();
  }

  pub fn add_locked(&mut self, locked_amount: &BigDecimal) {
    self.locked += locked_amount;
    self.upd_at = now();
  }
  
  pub fn sub_locked(&mut self, locked_amount: &BigDecimal) {
    self.locked -= locked_amount;
    self.upd_at = now();
  }

  pub fn free(&self) -> BigDecimal {
    self.free.clone()
  }

  pub fn locked(&self) -> BigDecimal {
    self.locked.clone()
  }
}
