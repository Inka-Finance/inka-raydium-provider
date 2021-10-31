//! Instruction types

#![allow(clippy::too_many_arguments)]

use crate::error::AmmError;
use crate::state::{Fees, AmmParams};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar,
};
use std::convert::TryInto;
use std::mem::size_of;
use arrayref::{array_ref};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InitializeInstruction {
    /// nonce used to create valid program address
    pub nonce: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MonitorStepInstruction {
    /// max value of plan/new/cancel orders
    pub plan_order_limit: u16,
    pub place_order_limit: u16,
    pub cancel_order_limit: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DepositInstruction {
    /// Pool token amount to transfer. token_a and token_b amount are set by
    /// the current exchange rate and size of the pool
    pub max_coin_amount: u64,
    pub max_pc_amount: u64,
    pub base_side: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WithdrawInstruction {
    /// Pool token amount to transfer. token_a and token_b amount are set by
    /// the current exchange rate and size of the pool
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WithdrawTransferInstruction {
    pub limit: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SetParamsInstruction {
    pub param: u8,
    pub value: Option<u64>,
    pub new_pubkey: Option<Pubkey>,
    pub fees: Option<Fees>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WithdrawSrmInstruction {
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwapInstruction {
    // SOURCE amount to transfer, output to DESTINATION is based on the exchange rate
    pub amount_in: u64,
    /// Minimum amount of DESTINATION token to output, prevents excessive slippage
    pub minimum_amount_out: u64,
}

/// Instructions supported by the AmmInfo program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AmmInstruction {

    ///   Deposit some tokens into the pool.  The output is a "pool" token representing ownership
    ///   into the pool. Inputs are converted to the current ratio.
    ///
    ///   0. `[]` Raydium Program id
    ///   1. `[]` Spl Token program id
    ///   2. `[writable]` amm Account
    ///   3. `[]` $authority
    ///   4. `[]` amm open_orders Account
    ///   5. `[writable]` amm target_orders Account. To store plan orders infomations.
    ///   6. `[writable]` pool lp mint address. Must be empty, owned by $authority.
    ///   7. `[writable]` pool_token_coin $authority can transfer amount,
    ///   8. `[writable]` pool_token_pc $authority can transfer amount,
    ///   9. `[]` serum market Account. serum_dex program is the owner.
    ///   10. `[writable]` user coin token Base Account to deposit into.
    ///   11. `[writable]` user pc token Base Account to deposit into.
    ///   12. `[writable]` user lp token. To deposit the generated tokens, user is the owner.
    ///   13. '[signer]` user owner Account
    ///   14. '[writable]` fee receiver Account
    Deposit(DepositInstruction),

    /// Swap coin or pc from pool
    ///
    ///   0. `[]` Raydium Program id
    ///   1. `[]` Spl Token program id
    ///   2. `[writable]` amm Account
    ///   3. `[]` $authority
    ///   4. `[writable]` amm open_orders Account
    ///   5. `[writable]` amm target_orders Account
    ///   6. `[writable]` pool_token_coin Amm Account to swap FROM or To,
    ///   7. `[writable]` pool_token_pc Amm Account to swap FROM or To,
    ///   8. `[]` serum dex program id
    ///   9. `[writable]` serum market Account. serum_dex program is the owner.
    ///   10. `[writable]` bids Account
    ///   11. `[writable]` asks Account
    ///   12. `[writable]` event_q Account
    ///   13. `[writable]` coin_vault Account
    ///   14. `[writable]` pc_vault Account
    ///   15. `[]` vault_signer Account
    ///   16. `[writable]` user source token Account. user Account to swap from.
    ///   17. `[writable]` user destination token Account. user Account to swap to.
    ///   18. `[singer]` user owner Account
    ///   19. `[writable]` fee receiver Account
    Swap(SwapInstruction),
}

impl AmmInstruction {
    /// Unpacks a byte buffer into a [AmmInstruction](enum.AmmInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(AmmError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (amount_in, rest) = Self::unpack_u64(rest)?;
                let (minimum_amount_out, _rest) = Self::unpack_u64(rest)?;
                Self::Swap(SwapInstruction{amount_in, minimum_amount_out})
            },
            1  => {
                let (max_coin_amount, rest) = Self::unpack_u64(rest)?;
                let (max_pc_amount, rest) = Self::unpack_u64(rest)?;
                let (base_side, _rest) = Self::unpack_u64(rest)?;
                Self::Deposit(DepositInstruction{ max_coin_amount, max_pc_amount, base_side })
            }
            _ => return Err(AmmError::InvalidInstruction.into()),
        })
    }

    fn unpack_u8(input: &[u8]) -> Result<(u8, &[u8]), ProgramError> {
        if input.len() >= 1 {
            let (amount, rest) = input.split_at(1);
            let amount = amount
                .get(..1)
                .and_then(|slice| slice.try_into().ok())
                .map(u8::from_le_bytes)
                .ok_or(AmmError::InvalidInstruction)?;
            Ok((amount, rest))
        } else {
            Err(AmmError::InvalidInstruction.into())
        }
    }

    fn unpack_u16(input: &[u8]) -> Result<(u16, &[u8]), ProgramError> {
        if input.len() >= 2 {
            let (amount, rest) = input.split_at(2);
            let amount = amount
                .get(..2)
                .and_then(|slice| slice.try_into().ok())
                .map(u16::from_le_bytes)
                .ok_or(AmmError::InvalidInstruction)?;
            Ok((amount, rest))
        } else {
            Err(AmmError::InvalidInstruction.into())
        }
    }

    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), ProgramError> {
        if input.len() >= 8 {
            let (amount, rest) = input.split_at(8);
            let amount = amount
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(AmmError::InvalidInstruction)?;
            Ok((amount, rest))
        } else {
            Err(AmmError::InvalidInstruction.into())
        }
    }

    /// Packs a [AmmInstruction](enum.AmmInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match &*self {
            Self::Swap(SwapInstruction{amount_in, minimum_amount_out}) => {
                buf.push(9);
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&minimum_amount_out.to_le_bytes());
            }
            Self::Deposit(DepositInstruction{ max_coin_amount, max_pc_amount, base_side }) => {
                buf.push(3);
                buf.extend_from_slice(&max_coin_amount.to_le_bytes());
                buf.extend_from_slice(&max_pc_amount.to_le_bytes());
                buf.extend_from_slice(&base_side.to_le_bytes());
            }
        }
        Ok(buf)
    }
}

/// Creates a 'deposit' instruction.
pub fn deposit(
    program_id: &Pubkey,
    amm_id: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_target_orders: &Pubkey,
    lp_mint_address: &Pubkey,
    pool_coin_token_account: &Pubkey,
    pool_pc_token_account: &Pubkey,
    serum_market: &Pubkey,
    user_coin_token_account: &Pubkey,
    user_pc_token_account: &Pubkey,
    user_lp_token_account: &Pubkey,
    user_owner: &Pubkey,

    max_coin_amount: u64,
    max_pc_amount: u64,
    base_side: u64,
) -> Result<Instruction, ProgramError> {
    let data = AmmInstruction::Deposit(DepositInstruction{ max_coin_amount, max_pc_amount, base_side }).pack()?;

    let accounts = vec![
        // spl token
        AccountMeta::new_readonly(spl_token::id(), false),
        // amm
        AccountMeta::new(*amm_id, false),
        AccountMeta::new_readonly(*amm_authority, false),
        AccountMeta::new_readonly(*amm_open_orders, false),
        AccountMeta::new(*amm_target_orders, false),
        AccountMeta::new(*lp_mint_address, false),
        AccountMeta::new(*pool_coin_token_account, false),
        AccountMeta::new(*pool_pc_token_account, false),
        // serum
        AccountMeta::new_readonly(*serum_market, false),
        // user
        AccountMeta::new(*user_coin_token_account, false),
        AccountMeta::new(*user_pc_token_account, false),
        AccountMeta::new(*user_lp_token_account, false),
        AccountMeta::new_readonly(*user_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates a 'swap' instruction.
pub fn swap(
    program_id: &Pubkey,
    amm_id: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_target_orders: &Pubkey,
    pool_coin_token_account: &Pubkey,
    pool_pc_token_account: &Pubkey,
    serum_program_id: &Pubkey,
    serum_market: &Pubkey,
    serum_bids: &Pubkey,
    serum_asks: &Pubkey,
    serum_event_queue: &Pubkey,
    serum_coin_vault_account: &Pubkey,
    serum_pc_vault_account: &Pubkey,
    serum_vault_signer: &Pubkey,
    uer_source_token_account: &Pubkey,
    uer_destination_token_account: &Pubkey,
    user_source_owner: &Pubkey,

    amount_in: u64,
    minimum_amount_out: u64,
) -> Result<Instruction, ProgramError> {
    let data = AmmInstruction::Swap(SwapInstruction{ amount_in, minimum_amount_out }).pack()?;

    let accounts = vec![
        // spl token
        AccountMeta::new_readonly(spl_token::id(), false),
        // amm
        AccountMeta::new(*amm_id, false),
        AccountMeta::new_readonly(*amm_authority, false),
        AccountMeta::new(*amm_open_orders, false),
        AccountMeta::new(*amm_target_orders, false),
        AccountMeta::new(*pool_coin_token_account, false),
        AccountMeta::new(*pool_pc_token_account, false),
        // serum
        AccountMeta::new_readonly(*serum_program_id, false),
        AccountMeta::new(*serum_market, false),
        AccountMeta::new(*serum_bids, false),
        AccountMeta::new(*serum_asks, false),
        AccountMeta::new(*serum_event_queue, false),
        AccountMeta::new(*serum_coin_vault_account, false),
        AccountMeta::new(*serum_pc_vault_account, false),
        AccountMeta::new_readonly(*serum_vault_signer, false),
        // user
        AccountMeta::new(*uer_source_token_account, false),
        AccountMeta::new(*uer_destination_token_account, false),
        AccountMeta::new_readonly(*user_source_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Unpacks a reference from a bytes buffer.
/// TODO actually pack / unpack instead of relying on normal memory layout.
pub fn unpack<T>(input: &[u8]) -> Result<&T, ProgramError> {
    if input.len() < size_of::<u8>() + size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    #[allow(clippy::cast_ptr_alignment)]
    let val: &T = unsafe { &*(&input[1] as *const u8 as *const T) };
    Ok(val)
}