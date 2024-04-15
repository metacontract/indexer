use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::json;
use reqwest::Client;


pub struct EthCall;

impl EthCall {
    pub async fn get_values_by_slots<'a>(slots: &HashMap<String, &'a str>, network: &'a str, contract_address: &'a str, contract_code: &'a str) -> Result<HashMap<String, &'a str>, Box<dyn std::error::Error>> {
        let chain_list = Self::get_chain_list();
        let api_url = "http://127.0.0.1:8545";

        let mut data = String::new();
        for (_, slot) in slots {
            data.push_str(slot);
        }

        let overrides = json!({
            contract_address: {
                "code": contract_code
            }
        });

        let gas_price = "0x45c77"; // got by gasEstimation

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [
                {
                    "to": contract_address,
                    "data": data,
                    "gas": "0x4C4B40", // 5,000,000 gas (less than block gas limit)
                    "gasPrice": gas_price,
                    "value": "0x0"
                },
                "latest", // or any other block number or tag
                overrides
            ],
            "id": chain_list[network]
        });

        let client = Client::new();
        let response = client
            .post(api_url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;

        let response_body: serde_json::Value = response.json().await?;
        let result = response_body["result"].as_str().unwrap();

        // Parse the result and return the values as a HashMap mapping EDFS to value
        let mut values: HashMap<String, String> = HashMap::new();
        let mut index = 0;
        for (edfs, _) in slots {
            let value = &result[index..index + 64];
            values.insert(edfs.clone(), &value.to_string());
            index += 64;
        }

        Ok(values)
    }

    pub fn get_chain_list() -> HashMap<String, i32> {
        let file_path = Path::new("chainIds.json");
        let file_content = fs::read_to_string(file_path).expect("Unable to read file");
        let id_to_network: HashMap<String, i32> = serde_json::from_str(&file_content).expect("Unable to parse JSON");
        
        id_to_network
            .iter()
            .map(|(k, v)| (v.to_string(), *k))
            .collect()
    }
}