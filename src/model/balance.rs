use bigdecimal::BigDecimal;


#[derive(Clone, Debug)]
pub struct Balance {
  free:   BigDecimal,
  locked: BigDecimal,
}

impl Balance {
  pub fn new(free: BigDecimal, locked: BigDecimal) -> Self {
    Self { free, locked }
  }

  pub fn default() -> Self {
    Self { free: BigDecimal::from(0), locked: BigDecimal::from(0) }
  }

  pub fn add_free(&mut self, free_amount: &BigDecimal) {
    self.free += free_amount;
  }

  pub fn sub_free(&mut self, free_amount: &BigDecimal) {
    self.free -= free_amount;
  }

  pub fn add_locked(&mut self, locked_amount: &BigDecimal) {
    self.locked += locked_amount;
  }
  
  pub fn sub_locked(&mut self, locked_amount: &BigDecimal) {
    self.locked -= locked_amount;
  }

  pub fn free(&self) -> BigDecimal {
    self.free.clone()
  }

  pub fn locked(&self) -> BigDecimal {
    self.locked.clone()
  }
}
