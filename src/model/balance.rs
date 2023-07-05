use bigdecimal::BigDecimal;

use crate::{library::common::now, balance_entity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Balance {
  free:   BigDecimal,
  locked: BigDecimal,
  block_number: i64,
  upd_at: i64,
}

impl Balance {
  pub fn new(free: BigDecimal, locked: BigDecimal, block_number: i64) -> Self {
    Self { free, locked, upd_at: now(), block_number }
  }

  pub fn from(entity: balance_entity::Model) -> Self {
    Self { free: entity.free, locked: entity.locked, upd_at: entity.updated_at, block_number: entity.block_number }
  }

  // pub fn from_state(state: State) -> Self {
  //   Self { free: state.balance, locked: BigDecimal::default(), upd_at: now() }
  // }

  pub fn default() -> Self {
    Self { free: BigDecimal::from(0), locked: BigDecimal::from(0), upd_at: now(), block_number: 0 }
  }

  pub fn add_free(&mut self, free_amount: &BigDecimal, block_number: i64) {
    self.free += free_amount;
    self.block_number = block_number;
    self.upd_at = now();
  }

  pub fn sub_free(&mut self, free_amount: &BigDecimal, block_number: i64) {
    self.free -= free_amount;
    self.block_number = block_number;
    self.upd_at = now();
  }

  pub fn add_locked(&mut self, locked_amount: &BigDecimal, block_number: i64) {
    self.locked += locked_amount;
    self.block_number = block_number;
    self.upd_at = now();
  }
  
  pub fn sub_locked(&mut self, locked_amount: &BigDecimal, block_number: i64) {
    self.locked -= locked_amount;
    self.block_number = block_number;
    self.upd_at = now();
  }

  pub fn update_free(&mut self, free_amount: BigDecimal, block_number: u64) {
    self.free = free_amount;
    self.block_number = block_number as i64;
    self.upd_at = now();
  }

  pub fn update_locked(&mut self, locked_amount: BigDecimal,  block_number: u64) {
    self.locked = locked_amount;
    self.block_number = block_number as i64;
    self.upd_at = now();
  }

  pub fn free(&self) -> BigDecimal {
    self.free.clone()
  }

  pub fn locked(&self) -> BigDecimal {
    self.locked.clone()
  }

  pub fn block_number(&self) -> i64 {
    self.block_number
  }

  // pub fn new_with_locked(locked: BigDecimal) -> Self {
  //   Self { free: BigDecimal::default(), locked, upd_at: now() }
  // }

  // pub fn new_with_free(free: BigDecimal) -> Self {
  //   Self { free, locked: BigDecimal::default(), upd_at: now() }
  // }
}
