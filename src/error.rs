use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AmmError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Invalid Input
    #[error("Invalid Input")]
    InvalidInput,
    /// Invalid fess
    #[error("Invalid fess")]
    InvalidFee,
/// Invalid status
    #[error("Invalid status")]
    InvalidStatus,
    #[error("Invalid status")]
    ConversionFailure
}

impl From<AmmError> for ProgramError {
    fn from(e: AmmError) -> Self {
        ProgramError::Custom(e as u32)
    }
}