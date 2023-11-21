#[tokio::test]
async fn test() {
    // WAL_INPUT
    //     .db
    //     .iter()
    //     .filter_map(Result::ok)
    //     .map(|(key, val)| State::from::<u64, HashMap<String, State>>(&key, &val))
    //     .sorted_by_key(|x| x.0)
    //     .into_iter()
    //     .for_each(|(key, _)| {
    //         println!("{key}");
    //     });

    // 마지막으로 체킹에 성공한 블록번호는 DB 에서 조회가능하므로 해당 블록넘버로 돌아갈수 있게 하는게 제일 적합.
    // let rollback_stage_number: u64 = 20000;

    // let mut free: Vec<(String, Balance)> = WAL_INPUT
    //     .db
    //     .iter()
    //     .filter_map(Result::ok)
    //     .map(|(key, val)| State::from(&key, &val))
    //     .collect::<BTreeMap<u64, HashMap<String, State>>>()
    //     .into_iter()
    //     .take_while(|(block_no, _)| block_no <= &rollback_stage_number)
    //     // .flat_map(|(_, stage_info)| stage_info.into_iter()) // 모든 stage_info 맵을 단일 스트림으로 변환
    //     .flat_map(|(_, v)| v.into_iter().collect::<Vec<(String, State)>>())
    //     .map(|(k, v)| (k, Balance::new_with_free(v.balance)))
    //     .collect();

    // let locked: Vec<(String, Balance)> = LockedBalanceStore::wal_input_db()
    //     .iter()
    //     .filter_map(Result::ok)
    //     .map(|(key, val)| State::from(&key, &val))
    //     .collect::<BTreeMap<u64, HashMap<String, State>>>()
    //     .into_iter()
    //     .take_while(|(block_no, _)| block_no <= &rollback_stage_number)
    //     // .flat_map(|(_, stage_info)| stage_info.into_iter()) // 모든 stage_info 맵을 단일 스트림으로 변환
    //     .flat_map(|(_, v)| v.into_iter().collect::<Vec<(String, State)>>())
    //     .map(|(k, v)| (k, Balance::new_with_locked(v.balance)))
    //     .into_iter()
    //     .collect();

    // free.extend(locked.clone());

    // let total_map: HashMap<String, Balance> = free.into_iter().into_group_map().into_iter().fold(
    //     HashMap::new(),
    //     |mut acc: HashMap<String, Balance>, (account, balances)| {
    //         let balance = balances
    //             .into_iter()
    //             .fold(Balance::default(), |mut acc, balance| {
    //                 if balance.free() == BigDecimal::default() {
    //                     acc.locked = balance.locked();
    //                 } else {
    //                     acc.free = balance.free();
    //                 }
    //                 acc
    //             });
    //         acc.insert(account, balance);
    //         acc
    //     },
    // );

    // check build state is valid
    // for (account, balance) in total_map {
    //     let res = ApiService::get_free_balance(&account).await.unwrap();
    //     if res.is_none() {
    //         println!("{account} 의 잔고가 존재 하지 않습니다.");
    //         continue;
    //     }
    //     let res = res.unwrap();
    //     let balance_info = res.get("LM").unwrap();
    //     println!(
    //         "{account} - {} - scan: {}, blc: {}",
    //         balance.free() == balance_info.total_amount,
    //         balance.free(),
    //         balance_info.total_amount
    //     );
    //     assert_eq!(balance.free(), balance_info.total_amount);

    //     let res = ApiService::get_locked_balance(&account).await.unwrap();
    //     if res.is_none() {
    //         println!("{account} 의 잔고가 존재 하지 않습니다.");
    //         continue;
    //     }
    //     let res = res.unwrap();
    //     let balance_info = res.get("LM").unwrap();
    //     println!(
    //         "{account} - {} - scan: {}, blc: {}",
    //         balance.locked() == balance_info.total_amount,
    //         balance.locked(),
    //         balance_info.total_amount
    //     );
    //     assert_eq!(balance.locked(), balance_info.total_amount);
    // }
}
