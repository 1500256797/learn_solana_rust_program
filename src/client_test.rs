#[cfg(test)]
mod test {
    use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
    use solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;
    #[test]
    pub fn client_test() {
        // Program ID (replace with your actual program ID)
        let program_id = Pubkey::from_str("CaDTBCo9DUTVT8AT3MB4taKBV6fXvQbPuWm4TgKFVMtZ").unwrap();

        // Connect to the Solana devnet
        let rpc_url = String::from("http://127.0.0.1:8899");
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // Generate a new keypair for the payer
        let payer = Keypair::new();

        // Request airdrop
        let airdrop_amount = 100_000_000_000; // 100 SOL
        let signature = client
            .request_airdrop(&payer.pubkey(), airdrop_amount)
            .expect("Failed to request airdrop");

        // Wait for airdrop confirmation
        loop {
            let confirmed = client.confirm_transaction(&signature).unwrap();
            if confirmed {
                break;
            }
        }

        // Create the instruction
        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: vec![],
        };

        // Add the instruction to new transaction
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], client.get_latest_blockhash().unwrap());

        // Send and confirm the transaction

        // Send and confirm the transaction
        match client.send_and_confirm_transaction_with_spinner(&transaction) {
            Ok(signature) => {
                println!("Transaction Signature: {}", signature);

                // 获取交易日志
                let transaction_status = client.get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: None,
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: Some(0),
                    },
                );

                if let Ok(tx_status) = transaction_status {
                    let logs = tx_status.transaction.meta.unwrap().log_messages.unwrap();
                    println!("Program Logs:");
                    for log in logs {
                        println!("  {}", log);
                    }
                }
            }
            Err(err) => eprintln!("Error sending transaction: {}", err),
        }
    }
}

fn main() {
    println!("Hello, world!");
}
