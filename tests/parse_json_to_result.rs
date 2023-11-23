#[cfg(test)]
mod tests {
    use lmscan_agent::library::common::parse_from_json_str;
    use lmscan_agent::transaction::TransactionWithResult;

    #[test]
    fn parse_json() {
        let json = r#"{"signedTx":{"sig":{"sig":{"v":27,"r":"a34df11d75d9ff173c28c11b18707cc3af3f9d6ff4867927ea158ad1f855caa7","s":"398587057fa59178f521593dce810703b91b716e677e659b3778548cd5d0aee3"},"account":"4b49c1ad5c1973b49f4fb131bdfddc314bf9a957"},"value":{"TokenTx":{"TransferFungibleToken":{"networkId":1000,"createdAt":"2023-05-09T01:50:13Z","tokenDefinitionId":"LM","inputs":["5f697f88dbe1707fae31894181bd2c6caa96e167e61bfcb446014bd9ba8a7d64","6db51aeee2eef45a305ff6cf46c46586f4b157a0c4f20c2502ee46e0140a1063","87122e6a07144aabaca35bcdf54d6affab47b3df2a5de5b66db241cace687912"],"outputs":{"a33872f06008d878e033c2c8aa7c084280cb362d":30000000000000000000,"4b49c1ad5c1973b49f4fb131bdfddc314bf9a957":277816019685259999999980000000,"eth-gateway":500000000000000000000},"memo":null}}}},"result":null}"#;

        let res = parse_from_json_str::<TransactionWithResult>(&json);
        println!("{:?}", res);
        println!("{:?}", res.result);
        println!("{:?}", res.signed_tx);
        println!("{:?}", res.signed_tx.value);
    }
}
