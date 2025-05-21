use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Hello, world!");
    Ok(())
}

#[cfg(test)]
mod test {
    use solana_program_test::*;
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Signer, transaction::Transaction,
    };

    #[tokio::test]
    async fn test_process_instruction() {
        let program_id = Pubkey::new_unique();
        // include program by name
        let mut program_test = ProgramTest::default();
        program_test.add_program("learn_solana_program", program_id, None);
        let (banks_client, payer, recent_blockhash) = program_test.start().await;
        // create a instruction data
        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: vec![],
        };

        // createt tx
        let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));

        // sign tx
        tx.sign(&[&payer], recent_blockhash);

        // tx res
        let tx_res = banks_client.process_transaction(tx).await;
        assert!(tx_res.is_ok());
    }
}
