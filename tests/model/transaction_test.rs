use lmscan_agent::model::transaction::*;

#[test]
pub fn transaction_test() {
    let no_result_json = r#"
    {
      "signedTx": {
        "sig": {
          "sig": {
            "v": 28,
            "r": "549091536c3628a6c7a7c95988cdd863a85bb2a8d42de28016bfb4523359252d",
            "s": "67750967063706d08b53f7079134207ded2b2f7039b01d50b96d551bf0a51bae"
          },
          "account": "playnomm"
        },
        "value": {
          "TokenTx": {
            "TransferNFT": {
              "networkId": 1000,
              "createdAt": "2023-02-24T05:10:42Z",
              "definitionId": "202302061200440725",
              "tokenId": "2023020612004400000000087",
              "input": "64c203e4019667ef8c26909c7d717f0b25bac405e8b44e57e5adf38853b0b67d",
              "output": "f9ff65d52bccb9c60f581f7bf5a61c364848b717",
              "memo": "Random Box Reveal"
            }
          }
        }
      },
      "result": null
    }
  "#;
    let json = r#"
  [
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "22c14ac6fbdce52c256640f1e36851ef901ea1b5cfebc3a430283a89df99bc11",
          "s" : "3474ebcc861c2d31a60d363356c4c89c196d450432b33bedadfb94d66edf2ffd"
        },
        "account" : "alice"
      },
      "value" : {
        "AccountTx" : {
          "UpdateAccount" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:00:00Z",
            "account" : "alice",
            "ethAddress" : "0xefD277f6da7ac53e709392044AE98220Df142753",
            "guardian" : null
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "495c3bcc143eea328c11b7ec55069dd4fb16c26463999f9dbc085094c3b59423",
          "s" : "707a75e433abd208cfb76d4e0cdbc04b1ce2389e3a1f866348ef2e3ea5785e93"
        },
        "account" : "alice"
      },
      "value" : {
        "AccountTx" : {
          "CreateAccount" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:00:00Z",
            "account" : "alice",
            "ethAddress" : null,
            "guardian" : null
          }
        }
      }
    },
    {
      "sig": {
        "sig": {
          "v": 27,
          "r": "816df20e4ff581fd2056689b48be73cca29e4f81977e5c42754e598757434c51",
          "s": "4e43aef8d836e79380067365cd7a4a452df5f52b73ec78463bdc7cdea2e11ca0"
        },
        "account": "alice"
      },
      "value": {
        "AccountTx": {
          "AddPublicKeySummaries": {
            "networkId": 1000,
            "createdAt": "2020-05-22T09:00:00Z",
            "account": "alice",
            "summaries": {
              "5b6ed47b96cd913eb938b81ee3ea9e7dc9affbff": "another key"
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "aab6f7ccc108b8e75601c726d43270c1a60f38f830136dfe293a2633dc86a0dd",
          "s" : "3cc1b610df7a421f9ae560853d5f07005a20c6ad225a00861a76e5e91aa183c0"
        },
        "account" : "alice"
      },
      "value" : {
        "GroupTx" : {
          "CreateGroup" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "groupId" : "mint-group",
            "name" : "mint group",
            "coordinator" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "2dd00a2ebf07ff2d09d6e9bcd889ddc775c17989827e3e19b5e8d1744c021466",
          "s" : "05bd60fef3d45463e22e5c157c814a7cbd1681410b67b0233c97ce7116d60729"
        },
        "account" : "alice"
      },
      "value" : {
        "GroupTx" : {
          "AddAccounts" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "groupId" : "mint-group",
            "accounts" : [
              "alice",
              "bob"
            ]
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "ce2b48b7da96eef22a2b92170fb81865adb99cbcae99a2b81bb7ce9b4ba990b6",
          "s" : "35a708c9ffc1b7ef4e88389255f883c96e551a404afc4627e3f6ca32a617bae6"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DefineToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:01:00Z",
            "definitionId" : "test-token",
            "name" : "test-token",
            "symbol" : "TT",
            "minterGroup" : "mint-group",
            "nftInfo" : {
              "Some" : {
                "value" : {
                  "minter" : "alice",
                  "rarity" : {
                    "LGDY" : 8,
                    "UNIQ" : 4,
                    "EPIC" : 2,
                    "RARE" : 1
                  },
                  "dataUrl" : "https://www.playnomm.com/data/test-token.json",
                  "contentHash" : "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                }
              }
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "76fb1b3be81101638c9ce070628db035ad7d86d3363d664da0c5afe254494e90",
          "s" : "7ffb1c751fe4f5341c75341e4a51373139a7f730a56a08078ac89b6e1a77fc76"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "MintFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-05-22T09:01:00Z",
            "definitionId" : "test-token",
            "outputs" : {
              "alice" : 100
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "0a914259cc0e8513512ea6356fc3056efe104e84756cf23a6c1c1aff7a580613",
          "s" : "71a15b331b9e7337a018b442ee978a15f0d86e71ca53d2f54a9a8ccb92646cf9"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "MintNFT" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-08T09:00:00Z",
            "tokenDefinitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "rarity" : "EPIC",
            "dataUrl" : "https://d3j8b1jkcxmuqq.cloudfront.net/temp/collections/TEST_NOMM4/NFT_ITEM/F7A92FB1-B29F-4E6F-BEF1-47C6A1376D68.jpg",
            "contentHash" : "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "output" : "alice"
          }
        }
      }
    }
    ,
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "09a5f46d29bd8598f04cb6db32627aadd562e30e181135c2898594080db6aa79",
          "s" : "340abd1b6618d3bbf4b586294a4f902942f597672330563a43591a14be0a6504"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "TransferFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "tokenDefinitionId" : "test-token",
            "inputs" : [
              "a3f35adb3d5d08692a7350e61aaa28da992a4280ad8e558953898ef96a0051ca"
            ],
            "outputs" : {
              "bob" : 10,
              "alice" : 90
            },
            "memo" : "transfer from alice to bob"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "c443ed5eda3d484bcda7bf77f030d3f6c20e4130d9bc4e03ca75df3074b40239",
          "s" : "2e7a19f1baee2099ccbef500e7ceb03c5053957a55085ef52b21c022c43242d9"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "TransferNFT" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "6040003b0020245ce82f352bed95dee2636442efee4e5a15ee3911c67910b657",
            "output" : "bob",
            "memo" : null
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "8d438670820bb788f0ef7106aa55c5fa2fa9c898eaded4d92f29d3c21a99c127",
          "s" : "1545783ca442a5ae2fdd347c79286a1c62256cd91ac76cb392f28dc190ac9c8a"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "EntrustFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2022-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "amount" : 1000,
            "inputs" : [
              "a3f35adb3d5d08692a7350e61aaa28da992a4280ad8e558953898ef96a0051ca"
            ],
            "to" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "05705f380f7a7fbad853094f69ff1527703476be30d2ac19f90a24a7900100c0",
          "s" : "37fac4695829b188ebe3d8238259a212ba52588c4593a51ef81631ab9ab90581"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "EntrustNFT" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-09T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "6040003b0020245ce82f352bed95dee2636442efee4e5a15ee3911c67910b657",
            "to" : "alice"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "fb6c99c0e26da04e8dc0855ea629708a17a8deabfabb5a488ba9faa001c4a31f",
          "s" : "7de70d3fd15176451e46856af2dbedf05e58d7cfc0bfb0e0fac1b6d06550f5d3"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DisposeEntrustedFungibleToken" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-10T09:00:00Z",
            "definitionId" : "test-token",
            "inputs" : [
              "45df6a88e74ea44f2d759251fed5a3c319e7cf9c37fafa7471418fec7b26acce"
            ],
            "outputs" : {
              "bob" : 1000
            }
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 28,
          "r" : "a03080b98925010e241783482e83a5fdfc25343406564a4e3fc4e6b2535657d3",
          "s" : "1de0ede5ebeba4aea455094ac1b58fc24ad943f0a5422a93f60a4f2b8b59b982"
        },
        "account" : "alice"
      },
      "value" : {
        "TokenTx" : {
          "DisposeEntrustedNFT" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-10T09:00:00Z",
            "definitionId" : "test-token",
            "tokenId" : "2022061710000513118",
            "input" : "10cb0802f3dfc85abb502bad260120a424fc583016db84d384904c1c0a580955",
            "output" : "bob"
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "d4b2d1cfe009e0e5b6dea67779fd898a7f1718e7b1869b5b36b6daacc68e88f6",
          "s" : "42d8c69e964109ceab5996abdbc59d53661904e6b56337599e9c5beebe665d51"
        },
        "account" : "alice"
      },
      "value" : {
        "RewardTx" : {
          "RegisterDao" : {
            "networkId" : 1000,
            "createdAt" : "2020-06-09T09:00:00Z",
            "groupId" : "sample-dao-group-id",
            "daoAccountName" : "sample-dao-group-account",
            "moderators" : [
              "alice"
            ]
          }
        }
      }
    },
    {
      "sig" : {
        "sig" : {
          "v" : 27,
          "r" : "95aff6586d03fa7c66165d9bb49f2a2fd54650f2776c728401c664622d5e2d4c",
          "s" : "2cff82c55822d3266add84ea5853dbc86cf47f24e5787080b76e58681477ba09"
        },
        "account" : "alice"
      },
      "value" : {
        "RewardTx" : {
          "RecordActivity" : {
            "networkId" : 2021,
            "createdAt" : "2023-01-10T18:01:00Z",
            "timestamp" : "2023-01-09T09:00:00Z",
            "userActivity" : {
              "bob" : [
                {
                  "point" : 3,
                  "description" : "like"
                }
              ],
              "carol" : [
                {
                  "point" : 3,
                  "description" : "like"
                }
              ]
            },
            "tokenReceived" : {
              "text-20230109-0000" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ],
              "text-20230109-0001" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ],
              "text-20230109-0002" : [
                {
                  "point" : 2,
                  "description" : "like"
                }
              ]
            }
          }
        }
      }
    }
  ]
  "#;

    let res = TransactionWithResult::from(no_result_json);
    assert_eq!(
        res,
        Some(TransactionWithResult {
            signed_tx: SignedTx {
                sig: AccountSignature {
                    sig: Signature {
                        v: 28,
                        r: "549091536c3628a6c7a7c95988cdd863a85bb2a8d42de28016bfb4523359252d"
                            .to_string(),
                        s: "67750967063706d08b53f7079134207ded2b2f7039b01d50b96d551bf0a51bae"
                            .to_string()
                    },
                    account: "playnomm".to_string()
                },
                value: Transaction::TokenTx(token_transaction::TokenTx::TransferNft(
                    token_transaction::TransferNft {
                        network_id: 1000,
                        created_at: "2023-02-24T05:10:42Z".to_string(),
                        definition_id: "202302061200440725".to_string(),
                        token_id: "2023020612004400000000087".to_string(),
                        input: "64c203e4019667ef8c26909c7d717f0b25bac405e8b44e57e5adf38853b0b67d"
                            .to_string(),
                        output: "f9ff65d52bccb9c60f581f7bf5a61c364848b717".to_string(),
                        memo: Some("Random Box Reveal".to_string())
                    }
                ))
            },
            result: None
        })
    );
}
