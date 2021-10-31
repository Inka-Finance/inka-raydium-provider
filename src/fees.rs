
//! All fee information, to be used for validation currently

use crate::error::SwapError;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};
use std::convert::TryFrom;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fees {
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub deposit_fee_numerator: u64,
    pub deposit_fee_denominator: u64,
}

/// Helper function for calculating swap fee
pub fn calculate_fee(
    token_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;
        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee)
        }
    }
}

fn validate_fraction(numerator: u64, denominator: u64) -> Result<(), SwapError> {
    if denominator == 0 && numerator == 0 {
        Ok(())
    } else if numerator >= denominator {
        Err(SwapError::InvalidFee)
    } else {
        Ok(())
    }
}

impl Fees {
    pub fn trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::try_from(self.trade_fee_numerator).ok()?,
            u128::try_from(self.trade_fee_denominator).ok()?,
        )
    }

    pub fn deposit_fee(&self, trading_tokens: u128) -> Option<u128> {
        calculate_fee(
            trading_tokens,
            u128::try_from(self.deposit_fee_numerator).ok()?,
            u128::try_from(self.deposit_fee_denominator).ok()?,
        )
    }

    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<(), SwapError> {
        validate_fraction(self.trade_fee_numerator, self.trade_fee_denominator)?;
        validate_fraction(
            self.deposit_fee_numerator,
            self.deposit_fee_denominator,
        )?;
        Ok(())
    }
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for Fees {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for Fees {}
impl Pack for Fees {
    const LEN: usize = 64;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 64];
        let (
            trade_fee_numerator,
            trade_fee_denominator,
            deposit_fee_numerator,
            deposit_fee_denominator,
        ) = mut_array_refs![output, 8, 8, 8, 8, 8, 8, 8, 8];
        *trade_fee_numerator = self.trade_fee_numerator.to_le_bytes();
        *trade_fee_denominator = self.trade_fee_denominator.to_le_bytes();
        *deposit_fee_numerator = self.deposit_fee_numerator.to_le_bytes();
        *deposit_fee_denominator = self.deposit_fee_denominator.to_le_bytes();
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Fees, ProgramError> {
        let input = array_ref![input, 0, 64];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            trade_fee_numerator,
            trade_fee_denominator,
            deposit_fee_numerator,
            deposit_fee_denominator,
        ) = array_refs![input, 8, 8, 8, 8, 8, 8, 8, 8];
        Ok(Self {
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            deposit_fee_numerator: u64::from_le_bytes(*deposit_fee_numerator),
            deposit_fee_denominator: u64::from_le_bytes(*deposit_fee_denominator),
        })
    }
}
