
  fn update_account_balance_info(&self, info: &mut HashMap<String, BigDecimal>) -> HashSet<String> {
    let mut updated_accouts = HashSet::new();
    let from_account = &self.signed_tx.sig.account;
    match &self.signed_tx.value {
        Transaction::RewardTx(tx) => match tx {
          RewardTx::OfferReward(t) => {
            // withdrawl from_account
            let sum: BigDecimal = t.outputs.values().sum();
            match info.get_mut(from_account) {
              Option::Some(value) => *value -= sum,
              None => {
                info.insert(from_account.to_owned(), -sum);
              },
            }
            updated_accouts.insert(from_account.clone());

            // deposit to_account
            for (to_account, new_value) in t.outputs.iter() {
              match info.get_mut(to_account) {
                Option::Some(value) => *value += new_value,
                None => {
                  info.insert(to_account.to_owned(), new_value.clone());
                },
              };
              updated_accouts.insert(to_account.clone());
            };
          },
        RewardTx::ExecuteReward(t) => {
          self.result.as_ref().map(|res| match res {
            TransactionResult::ExecuteRewardResult(res) => {
              // withdrawl from_account
              let sum: BigDecimal = res.outputs.values().sum();
              match info.get_mut(from_account) {
                Option::Some(value) => *value -= sum,
                None => {
                  info.insert(from_account.to_owned(), -sum);
                },
              }
              updated_accouts.insert(from_account.clone());

              // deposit to_account
              for (to_account, new_value) in res.outputs.iter() {
                match info.get_mut(to_account) {
                  Option::Some(value) => *value += new_value,
                  None => {
                    info.insert(to_account.to_owned(), new_value.clone());
                  },
                };
                updated_accouts.insert(to_account.clone());
              };
            },
            _ => (),
          });
        },
        _ => (),
      },
      Transaction::TokenTx(tx) => match tx {
        TokenTx::EntrustFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.amount.clone();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.clone(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          match info.get_mut(t.to.as_str()) {
            Option::Some(value) => *value += t.amount.clone(),
            None => {
              info.insert(t.to.clone(), t.amount.clone());
            },
          };
          updated_accouts.insert(t.to.clone());
        },
        TokenTx::TransferFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },
        TokenTx::MintFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },  
        TokenTx::DisposeEntrustedFungibleToken(t) => {
          // withdrawl from_account
          let sum: BigDecimal = t.outputs.values().sum();
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= sum,
            None => {
              info.insert(from_account.to_owned(), -sum);
            },
          }
          updated_accouts.insert(from_account.clone());

          // deposit to_account
          for (to_account, new_value) in t.outputs.iter() {
            match info.get_mut(to_account) {
              Option::Some(value) => *value += new_value,
              None => {
                info.insert(to_account.to_owned(), new_value.clone());
              },
            };
            updated_accouts.insert(to_account.clone());
          };
        },  
        TokenTx::BurnFungibleToken(t) => {
          match info.get_mut(from_account) {
            Option::Some(value) => *value -= t.amount.clone(),
            None => { 
              info.insert(from_account.clone(), BigDecimal::from(0));
            },
          }
        },
        _ => (),
      },
      _ => ()
    };
    updated_accouts
  }
