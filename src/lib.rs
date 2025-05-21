// new a counter program. user can increase or decrease the counter value.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
/// Total extra compute units used per compute_fn! call 409 CU
/// https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/programs/bpf_loader/src/syscalls/logging.rs#L70
/// https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/program-runtime/src/compute_budget.rs#L150
#[macro_export]
macro_rules! compute_fn {
    ($msg:expr=> $($tt:tt)*) => {
        ::solana_program::msg!(concat!($msg, " {"));
        ::solana_program::log::sol_log_compute_units();
        let res = { $($tt)* };
        ::solana_program::log::sol_log_compute_units();
        ::solana_program::msg!(concat!(" } // ", $msg));
        res
    };
}
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 105 cu
    compute_fn! { "calculate unpack cu " =>
        let instruction = CounterInstruction::unpack(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;
    }

    let instruction = CounterInstruction::unpack(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CounterInstruction::InitializeCounter { init_value } => {
            process_initialize_counter(program_id, accounts, init_value)?
        }
        CounterInstruction::Increment => increment_counter(program_id, accounts)?,
        CounterInstruction::Decrement => decrement_counter(program_id, accounts)?,
    };
    Ok(())
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct CounterAccount {
    counter: u64,
}

pub fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    init_value: u64,
) -> ProgramResult {
    // get account ordered by the account info
    let accounts_iter = &mut accounts.iter();
    // counter account will be created and initialized by the system program
    let counter_account = next_account_info(accounts_iter)?;
    // payer account will pay for the counter account rent
    let payer = next_account_info(accounts_iter)?;
    // system program will create the counter account
    let system_program = next_account_info(accounts_iter)?;

    // step2 alloc the space for the counter account and calculate the rent
    let account_space = 8;
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(account_space);

    // use system program to create the counter account

    // we need cpi to call the system program from our program
    invoke(
        &system_instruction::create_account(
            &payer.key,           // will pay for rent
            &counter_account.key, // create account at counter_account.key
            required_lamports,    // lamports
            account_space as u64, // space
            &program_id,          // set owner to program id
        ),
        &[
            payer.clone(),
            counter_account.clone(),
            system_program.clone(),
        ],
    )?;

    // initialize the counter account
    let data = CounterAccount {
        counter: init_value,
    };
    // get the account mut ref
    let mut account_data = &mut counter_account.data.borrow_mut()[..];
    data.serialize(&mut account_data)?;

    msg!("Counter account initialized");
    Ok(())
}

fn increment_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    // when increment , we dont need to pay for rent
    // verify the account owner is the program id only the specific program can increment the counter
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    // deserialize the counter account
    let mut counter_data = &mut counter_account.data.borrow_mut();

    // deserialize the counter data
    let mut counter = CounterAccount::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    counter.counter = counter
        .counter
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // serilaze the updated counter data
    counter.serialize(&mut &mut counter_data[..])?;
    msg!("Counter incremented to: {}", counter.counter);
    Ok(())
}

fn decrement_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // verify the account owner is the program id only the specific program can decrement the counter
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // get mut ref to the counter data
    let mut counter_data = &mut counter_account.data.borrow_mut();
    // deserialize the counter data
    let mut counter = CounterAccount::try_from_slice(&counter_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    counter.counter = counter
        .counter
        .checked_sub(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // serialize the updated counter data
    counter.serialize(&mut &mut counter_data[..])?;
    msg!("Counter decremented to: {}", counter.counter);
    Ok(())
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum CounterInstruction {
    InitializeCounter { init_value: u64 },
    Increment,
    Decrement,
}

impl CounterInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        match variant {
            0 => Ok(Self::InitializeCounter {
                init_value: u64::from_le_bytes(rest[..8].try_into().unwrap()),
            }),
            1 => Ok(Self::Increment),
            2 => Ok(Self::Decrement),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
    };
    #[tokio::test]
    async fn test_initialize_counter() {
        let program_id = Pubkey::new_unique();

        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "learn_solana_program",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;
        // Create a new keypair to use as the address for our counter account
        let counter_keypair = Keypair::new();
        let initial_value: u64 = 42;

        // step1 initialize the counter account
        println!("Initializing counter account...");
        // create initialize instruction
        let mut init_instruction_data = vec![0];
        init_instruction_data.extend_from_slice(&initial_value.to_le_bytes());

        let init_instruction = Instruction::new_with_bytes(
            program_id,
            &init_instruction_data,
            vec![
                AccountMeta::new(counter_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );
        // send the instruction to the program
        let transaction = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&payer.pubkey()),
            &[&payer, &counter_keypair],
            recent_blockhash,
        );
        // process the transaction
        let res = banks_client.process_transaction(transaction).await.unwrap();
        println!("Transaction result: {:?}", res);

        // check the counter account value
        // Check account data
        let account = banks_client
            .get_account(counter_keypair.pubkey())
            .await
            .expect("Failed to get counter account");

        if let Some(account_data) = account {
            let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
                .expect("Failed to deserialize counter data");
            assert_eq!(counter.counter, 42);
            println!(
                "✅ Counter initialized successfully with value: {}",
                counter.counter
            );
        }

        // increment the counter
        let increment_instruction = Instruction::new_with_bytes(
            program_id,
            &[1],
            vec![AccountMeta::new(counter_keypair.pubkey(), true)],
        );
        let transaction = Transaction::new_signed_with_payer(
            &[increment_instruction],
            Some(&payer.pubkey()),
            &[&payer, &counter_keypair],
            recent_blockhash,
        );
        let res = banks_client.process_transaction(transaction).await.unwrap();
        // get the counter account value
        let account = banks_client
            .get_account(counter_keypair.pubkey())
            .await
            .expect("Failed to get counter account");
        if let Some(account_data) = account {
            let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
                .expect("Failed to deserialize counter data");
            assert_eq!(counter.counter, 43);
            println!(
                "✅ Counter incremented successfully to: {}",
                counter.counter
            );
        }

        // decrement the counter
        let decrement_instruction = Instruction::new_with_bytes(
            program_id,
            &[2],
            vec![AccountMeta::new(counter_keypair.pubkey(), true)],
        );
        let transaction = Transaction::new_signed_with_payer(
            &[decrement_instruction],
            Some(&payer.pubkey()),
            &[&payer, &counter_keypair],
            recent_blockhash,
        );
        let res = banks_client.process_transaction(transaction).await.unwrap();
        // get the counter account value
        let account = banks_client
            .get_account(counter_keypair.pubkey())
            .await
            .expect("Failed to get counter account");
        if let Some(account_data) = account {
            let counter: CounterAccount = CounterAccount::try_from_slice(&account_data.data)
                .expect("Failed to deserialize counter data");
            assert_eq!(counter.counter, 42);
            println!(
                "✅ Counter decremented successfully to: {}",
                counter.counter
            );
        }
    }
}
