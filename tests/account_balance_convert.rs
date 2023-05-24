use std::{collections::{HashMap, HashSet}, str::FromStr, fs::File, path::Path, io::Write};

use bigdecimal::BigDecimal;
use itertools::Itertools;
use lmscan_agent::{block_state, library::common::parse_from_json_str, transaction::{TransactionWithResult, Job, Common}, block_entity, tx_state, model::balance_info::{BalanceInfo}, service::api_service::ApiService};
use dotenvy::{dotenv};
use sea_orm::{*, sea_query::Expr};
use lmscan_agent::tx_state::{Entity as TxState};

#[tokio::test]
async fn account_balance_convert() {
  dotenv().expect("Unable to load environment variables from .env file");
  // let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set.");
  // let ref db = db_connn(database_url).await;

  let mut output_file = File::create(Path::new("balance_local_build.txt"))
                                    // .append(true)
                                    // .open("")
                                    .expect("cannot open output file");
  // output_file.write(format!("address, blc_balance, scan_balance, equal, diff\n").as_bytes()).expect("write failed");
  // let result = build_saved_state_proc(db, HashMap::new(), &mut HashMap::new()).await;
  // output_file.write(format!("builded account total count: {}\n", result.keys().count()).as_bytes()).expect("write failed");

  // let scan_accounts = account_entity::Entity::find().all(db).await.unwrap();
  // output_file.write(format!("scan account total count: {}\n", scan_accounts.len()).as_bytes()).expect("write failed");

  //for scan_account in scan_accounts.into_iter() {
  let scan_addr = "8e39dcc13ebdb7e8eb3da92090e4058c44ec9ca7";
  //if let Some(scan_balance) = result.get(scan_addr) {
  // println!("{count} - scan {key} - {scan_balance}");
  match ApiService::get_account_balance(scan_addr).await {
    Ok(block_account_balance_opt) => {
      println!("1111111");

      let block_account_balance = block_account_balance_opt.unwrap();
      if let Some(block_info) = block_account_balance.get("LM") {
        println!("block_info.total_amount: {:?}", block_info.total_amount);
        let line = format!("{}, {:?}\n", 
                  scan_addr, 
                  block_info.total_amount,
                );
        output_file.write(line.as_bytes()).expect("write failed");
      }
    }
    Err(err) => println!("request err: {err}")
  }

  let res = r#"{"LM":{"totalAmount":80000000000000000000000000,"unused":{"6e2620f35bcfc890a878fb9479075a5295cc811e58aa9fb90154a6256db4ddaa":{"signedTx":{"sig":{"sig":{"v":27,"r":"3e648f131948128a250bc56c20b721126278273fa56a02a8e53ae08a4989fde1","s":"7c3a2bff742268cad4588c6dfcec19d43c71aaeac9463d3351ef0231b84feffd"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-03-24T12:31:19Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"7755302f21fd9d6896f7bbb86a697114311582c90803735ad220d4ddbf4c305c":{"signedTx":{"sig":{"sig":{"v":27,"r":"1abd01233bc54716916b663911b98b7dabd01615d4abe3d3f250528a9febda70","s":"69d9edec1277169407ce94e91ceee868316e528bfaaffd40c528554f3826317a"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-02-02T12:40:59Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"cb258e8745055736952cdab5048b2b7c4503cc7a7fa528f1180eeaa5de256d40":{"signedTx":{"sig":{"sig":{"v":28,"r":"18022de12f2ab5a42635c388482c4a2bf9af4236cc9cc2abea580b37c897b631","s":"6f27b0bcaa2d1f5ab0005a6f6b5622030e19ed0ec758dbe382c4b73208f6b8c0"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-03-24T12:38:20Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"90cf0f4bbd097f3cb472686526592cd6e33e2bba0e53ddcf7dc06df75eb6b53b":{"signedTx":{"sig":{"sig":{"v":27,"r":"11e820c844c0bfb2107093706d75a2871c44f3735c1d265c9e159744e36b217c","s":"53bc9ab27349d09c9d591f6e7b3ecfc4cdf42da596c3bf861406deed3f936d15"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-02-15T13:18:31Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"5238ecf172d9a4c6aedbe6a893b2131bb8efeb1212ee6990e317c4fc1d35a8c2":{"signedTx":{"sig":{"sig":{"v":27,"r":"612c7d8fb3e7f90dfeaae1eac0c27ceb2e45332dd6ca8a4907f94cd4b70f7ec1","s":"6681d12d415b320a39b12500d966e60ad3f2d123d6ce7aec260a764578f0e568"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-02-15T13:32:59Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"ac5f3c7484ebc8e38f88af340384f3f00ee4b122c9dab08f81fb205baa6b85ae":{"signedTx":{"sig":{"sig":{"v":27,"r":"8ae9959e1adedc14d488d5eeeedb14fe5f648073ecc6cfaa002da4c21ad0496c","s":"38fb03611ca0d7ca5001bae7f187e29b2b686b3b6db535a4967551ec07ca957e"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-02-13T11:23:55Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"7ee28e631604f0bcee06b97fd74919ac8fbaf23bb7e324edc6bdca6c9ddfab6c":{"signedTx":{"sig":{"sig":{"v":27,"r":"71f049142175bb1f62f42d30e032ba893197a23d2debf3fa84d237e7b15a96d6","s":"193a378080de6e1b1d0177c0cf2098b9da63e4da30b3d0c467fa99df291e1f85"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-03-24T12:39:52Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null},"77fa193fc89441db7517b54485c3fc9ad6c7951d2cf1e489260b8abe87c2e4ad":{"signedTx":{"sig":{"sig":{"v":27,"r":"f5b67a5a20424f21da94250100f0ea9993d719f1465b5f3b7faf2232397838b9","s":"5b5df1a55ea650e76c48851faa98ac3ae703ca8da2ef2e70bfbdf1e61ae39b24"},"account":"playnomm"},"value":{"TokenTx":{"MintFungibleToken":{"networkId":1000,"createdAt":"2023-02-06T17:02:03Z","definitionId":"LM","outputs":{"a3adf5c2606cf1986abfe9fdd07b49bce2bb889f":10000000000000000000000000}}}}},"result":null}}}}"#;
  let res: HashMap<String, BalanceInfo> = serde_json::from_str(res).unwrap();

  // println!("total_amount {:?}", res);

  let balance = BigDecimal::from_str("80000000000000000000000000").unwrap();

  println!("balance {:?}", balance);
}


async fn build_saved_state_proc(db: &DatabaseConnection, mut account_balance_info: HashMap<String, BigDecimal>, nft_owner_info: &mut HashMap<String, String>) -> HashMap<String, BigDecimal>{
  
  while let Some(block_states) = get_block_states_not_built_order_by_asc_limit(db).await  {
    let mut cloned_account_balance_info = account_balance_info.clone();
    let mut block_entities = vec![];
    let mut balance_updated_accounts = HashSet::new();

    let block_hashs = block_states.iter().map(|b|b.hash.clone()).collect::<Vec<String>>();
    let mut txs_in_block = get_tx_states_in_block_hashs(block_hashs, db).await;

    for block_state in block_states.iter() {
      let block = parse_from_json_str(block_state.json.as_str());
      block_entities.push(block_entity::Model::from(&block, block_state.hash.clone()));
      println!("{}", block.header.number);
      if let Some(tx_states_in_block) = txs_in_block.remove(&block_state.hash) {
        let iter = tx_states_in_block.iter().map(|state| (state, parse_from_json_str::<TransactionWithResult>(state.json.as_str())))
                                      .sorted_by_key(|(_, tx_res)| tx_res.signed_tx.value.created_at());

        for (tx_state, tx_res) in iter {
          balance_updated_accounts.extend(tx_res.update_account_balance_info(&mut cloned_account_balance_info).await);
          // transfered_nft_token_ids.extend(tx_res.update_nft_owner_info(nft_owner_info));
          
        }
      }
    }

    // let this_time_updated_nft_owners = extract_updated_nft_owners(&nft_owner_info, transfered_nft_token_ids);
    let this_time_updated_balance_accounts = extract_updated_balance_accounts(&cloned_account_balance_info, balance_updated_accounts);
    // let addresses = extract_addresses(additional_entity_store.get(&AdditionalEntityKey::CreateAccount));
    // let token_ids = extract_token_ids(additional_entity_store.get(&AdditionalEntityKey::CreateNftFile));
    match finish_all_block_states(block_states, db).await {
      true => account_balance_info = cloned_account_balance_info,
      false => panic!("finish_all_block_states error"),
    }
  } 
  account_balance_info
}


async fn get_block_states_not_built_order_by_asc_limit(db: &DatabaseConnection) -> Option<Vec<block_state::Model>> {
  block_state::Entity::find()
                      .filter(block_state::Column::IsBuild.eq(false))
                      .order_by_asc(block_state::Column::Number)
                      .paginate(db, BUILD_BATCH_UNIT).fetch_and_next().await.unwrap()
}
static BUILD_BATCH_UNIT: u64 = 50;

fn extract_updated_balance_accounts(account_balance_info: &HashMap<String, BigDecimal>, balanced_updated_accounts: HashSet<String>) -> HashMap<String, BigDecimal> {
  account_balance_info.iter()
    .filter(|(k, _)| balanced_updated_accounts.contains(*k))
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect()
}




async fn get_tx_states_in_block_hashs(block_hashs: Vec<String>, db: &DatabaseConnection) -> HashMap<String, Vec<tx_state::Model>> {

  TxState::find().filter(tx_state::Column::BlockHash.is_in(block_hashs))
                  .order_by_asc(tx_state::Column::EventTime)
                  .all(db).await.unwrap()
                  .into_iter()
                  .fold(HashMap::new(), |mut acc, tx| {
                    acc.entry(tx.block_hash.clone()).or_insert_with(Vec::new).push(tx);
                    acc
                  })
}

async fn finish_all_block_states(block_states: Vec<block_state::Model>, db: &DatabaseConnection) -> bool {
  if block_states.is_empty() { return true }
  if let Err(err) = block_state::Entity::update_many()
                                        .col_expr(block_state::Column::IsBuild, Expr::value(true))
                                        .filter(block_state::Column::Hash.is_in(block_states.iter().map(|b| b.hash.clone())
                                        .collect::<Vec<String>>()))
                                        .exec(db).await {
    return false;
  }
  true
}

