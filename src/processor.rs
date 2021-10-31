use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    instruction::{AccountMeta},
};
use crate::error::AmmError;
use crate::instruction::{AmmInstruction, SwapInstruction, DepositInstruction};
use std::convert::TryInto;
pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        let instruction = AmmInstruction::unpack(instruction_data)?;

        match instruction {
            AmmInstruction::Swap(SwapInstruction {amount_in, minimum_amount_out}) => {
                msg!("Instruction: SwapInstruction");
                Self::swap(accounts, amount_in, minimum_amount_out, program_id)
            },
            AmmInstruction::Deposit(DepositInstruction {max_coin_amount, max_pc_amount, base_side}) => {
                msg!("Instruction: DepositInstruction");
                Self::deposit(accounts, max_coin_amount, max_pc_amount, base_side, program_id)
            },
        }
    }

    fn deposit(
        accounts: &[AccountInfo],
        max_coin_amount: u64, 
        max_pc_amount: u64, 
        base_side: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let raydium_program_id = next_account_info(account_info_iter)?;

        let spl_token_program_id = next_account_info(account_info_iter)?;
        let amm_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let amm_open_orders = next_account_info(account_info_iter)?;
        let amm_target_orders = next_account_info(account_info_iter)?;
        let pool_lp_mint = next_account_info(account_info_iter)?;
        let pool_token_coin = next_account_info(account_info_iter)?;
        let pool_token_pc = next_account_info(account_info_iter)?;
        let serum_market = next_account_info(account_info_iter)?;
        let user_coin_token_base_account = next_account_info(account_info_iter)?;
        let user_pc_token_base_account = next_account_info(account_info_iter)?;
        let user_lp_token_base_account = next_account_info(account_info_iter)?;
        let user_owner_account = next_account_info(account_info_iter)?;
        let fee_receiver_first = next_account_info(account_info_iter)?;
        let fee_receiver_second = next_account_info(account_info_iter)?;
        let fee_first = max_coin_amount
            .checked_mul(1).unwrap()
            .checked_div(10).unwrap();

        let fee_second = max_pc_amount
            .checked_mul(1).unwrap()
            .checked_div(10).unwrap();

        let final_amount_coin = max_coin_amount.checked_sub(fee_first).unwrap();
        let final_amount_pc = max_pc_amount.checked_sub(fee_second).unwrap();

        let deposit_tx = crate::instruction::deposit(
            raydium_program_id.key,
            amm_account.key,
            authority.key,
            amm_open_orders.key,
            amm_target_orders.key,
            pool_lp_mint.key,
            pool_token_coin.key,
            pool_token_pc.key,
            serum_market.key,
            user_coin_token_base_account.key,
            user_pc_token_base_account.key,
            user_lp_token_base_account.key,
            user_owner_account.key,
            final_amount_coin, 
            final_amount_pc, 
            base_side,
        )?;
        invoke(
            &deposit_tx, 
            &[
                spl_token_program_id.clone(),
                amm_account.clone(),
                authority.clone(),
                amm_open_orders.clone(),
                amm_target_orders.clone(),
                pool_lp_mint.clone(),
                pool_token_coin.clone(),
                pool_token_pc.clone(),
                serum_market.clone(),
                user_coin_token_base_account.clone(),
                user_pc_token_base_account.clone(),
                user_lp_token_base_account.clone(),
                user_owner_account.clone(),
             ]
        )?;

        let tx_to_receive_first = spl_token::instruction::transfer(
            spl_token_program_id.key, 
            user_coin_token_base_account.key, 
            fee_receiver_first.key,
            user_owner_account.key,
            &[],
            fee_first,
        )?;

        let tx_to_receive_second = spl_token::instruction::transfer(
            spl_token_program_id.key, 
            user_pc_token_base_account.key, 
            fee_receiver_second.key,
            user_owner_account.key,
            &[],
            fee_second,
        )?;
        invoke(
            &tx_to_receive_first,
            &[
                user_coin_token_base_account.clone(),
                fee_receiver_first.clone(),
                user_owner_account.clone(),
            ],
        )?;
        invoke(
            &tx_to_receive_second,
            &[
                user_pc_token_base_account.clone(),
                fee_receiver_second.clone(),
                user_owner_account.clone(),
            ],
        )?;
        Ok(())
    }

    fn swap(
        accounts: &[AccountInfo],
        amount_in: u64, 
        minimum_amount_out: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        msg!("Swap initialized");
        let account_info_iter = &mut accounts.iter();
        let raydium_program_id = next_account_info(account_info_iter)?;

        let spl_token_program_id = next_account_info(account_info_iter)?;
        let amm_account = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let amm_open_orders = next_account_info(account_info_iter)?;
        let amm_target_orders = next_account_info(account_info_iter)?;
        let pool_token_coin = next_account_info(account_info_iter)?;
        let pool_token_pc = next_account_info(account_info_iter)?;
        let serum_dex_program_id = next_account_info(account_info_iter)?;
        let serum_market = next_account_info(account_info_iter)?;
        let serum_bids = next_account_info(account_info_iter)?;
        let serum_asks = next_account_info(account_info_iter)?;
        let serum_event_queue = next_account_info(account_info_iter)?;
        let serum_coin_vault_account = next_account_info(account_info_iter)?;
        let serum_pc_vault_account = next_account_info(account_info_iter)?;
        let serum_vault_signer = next_account_info(account_info_iter)?;
        let user_source_token_account = next_account_info(account_info_iter)?;
        let user_destination_token_account = next_account_info(account_info_iter)?;
        let user_owner_account = next_account_info(account_info_iter)?;
        let fee_receiver = next_account_info(account_info_iter)?;
        msg!("Swap Instruction");
        let instruction = crate::instruction::swap(
            raydium_program_id.key,
            amm_account.key,
            authority.key,
            amm_open_orders.key,
            amm_target_orders.key,
            pool_token_coin.key,
            pool_token_pc.key,
            serum_dex_program_id.key,
            serum_market.key,
            serum_bids.key,
            serum_asks.key,
            serum_event_queue.key,
            serum_coin_vault_account.key,
            serum_pc_vault_account.key,
            serum_vault_signer.key,
            user_source_token_account.key,
            user_destination_token_account.key,
            user_owner_account.key,
            amount_in,
            minimum_amount_out
        )?;
        msg!("Swap Invoke");
        invoke(
            &instruction,
            &[
                spl_token_program_id.clone(),
                amm_account.clone(),
                authority.clone(),
                amm_open_orders.clone(),
                amm_target_orders.clone(),
                pool_token_coin.clone(),
                pool_token_pc.clone(),
                serum_dex_program_id.clone(),
                serum_market.clone(),
                serum_bids.clone(),
                serum_asks.clone(),
                serum_event_queue.clone(),
                serum_coin_vault_account.clone(),
                serum_pc_vault_account.clone(),
                serum_vault_signer.clone(),
                user_source_token_account.clone(),
                user_destination_token_account.clone(),
                user_owner_account.clone()
            ],
        )?;
        let fee = minimum_amount_out
            .checked_mul(1).unwrap()
            .checked_div(10).unwrap();

        let tx_to_receive = spl_token::instruction::transfer(
            spl_token_program_id.key, 
            user_destination_token_account.key, 
            fee_receiver.key,
            user_owner_account.key,
            &[],
            fee,
        )?;
        invoke(
            &tx_to_receive,
            &[
                user_destination_token_account.clone(),
                fee_receiver.clone(),
                user_owner_account.clone(),
            ],
        )?;


        msg!("OKOKOKO");
        Ok(())
    }
}

fn to_u128(val: u64) -> Result<u128, AmmError> {
    val.try_into().map_err(|_| AmmError::ConversionFailure)
}

fn to_u64(val: u128) -> Result<u64, AmmError> {
    val.try_into().map_err(|_| AmmError::ConversionFailure)
}
