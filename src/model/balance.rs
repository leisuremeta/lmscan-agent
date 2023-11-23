use bigdecimal::BigDecimal;

use crate::{balance_entity, library::common::now, store::wal::State};

#[derive(Clone, Debug)]
pub struct Balance {
    pub free: BigDecimal,
    pub locked: BigDecimal,
    upd_at: i64,
}

impl Balance {
    pub fn new(free: BigDecimal, locked: BigDecimal) -> Self {
        Self {
            free,
            locked,
            upd_at: now(),
        }
    }

    pub fn from(entity: balance_entity::Model) -> Self {
        Self {
            free: entity.free,
            locked: entity.locked,
            upd_at: now(),
        }
    }

    pub fn from_state(state: State) -> Self {
        Self {
            free: state.balance,
            locked: BigDecimal::default(),
            upd_at: now(),
        }
    }

    pub fn default() -> Self {
        Self {
            free: BigDecimal::from(0),
            locked: BigDecimal::from(0),
            upd_at: now(),
        }
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

    pub fn new_with_locked(locked: BigDecimal) -> Self {
        Self {
            free: BigDecimal::default(),
            locked,
            upd_at: now(),
        }
    }

    pub fn new_with_free(free: BigDecimal) -> Self {
        Self {
            free,
            locked: BigDecimal::default(),
            upd_at: now(),
        }
    }
}
