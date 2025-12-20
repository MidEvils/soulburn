use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum SoulburnError {
    /// 0 - Error deserializing an account
    #[error("Error deserializing an account")]
    DeserializationError,
    /// 1 - Error serializing an account
    #[error("Error serializing an account")]
    SerializationError,
    /// 2 - Invalid program owner
    #[error("Invalid program owner. This likely mean the provided account does not exist")]
    InvalidProgramOwner,
    /// 3 - Invalid PDA derivation
    #[error("Invalid PDA derivation")]
    InvalidPda,
    /// 4 - Expected empty account
    #[error("Expected empty account")]
    ExpectedEmptyAccount,
    /// 5 - Expected non empty account
    #[error("Expected non empty account")]
    ExpectedNonEmptyAccount,
    /// 6 - Expected signer account
    #[error("Expected signer account")]
    ExpectedSignerAccount,
    /// 7 - Expected writable account
    #[error("Expected writable account")]
    ExpectedWritableAccount,
    /// 8 - Account mismatch
    #[error("Account mismatch")]
    AccountMismatch,
    /// 9 - Invalid account key
    #[error("Invalid account key")]
    InvalidAccountKey,
    /// 10 - Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow,
    /// 11 - Expected MPL Core Collection
    #[error("Expected MPL Core Collection")]
    ExpectedMplCoreCollection,
    /// 12 - Expected MPL Core Asset
    #[error("Expected MPL Core Asset")]
    ExpectedMplCoreAsset,
    /// 13 - Invalid core collection
    #[error("Invalid collection for asset")]
    InvalidCollectionForAsset,
    /// 14 - Invalid core collection
    #[error("Invalid owner for asset")]
    InvalidAssetOwner,
    /// 15 - Invalid end time
    #[error("Invalid end time, must be at least 1hr")]
    InvalidEndTime,
    /// 16 - Invalid remaining accounts
    #[error(
        "Remaning accounts must be in pairs of 2 and equal to the number of assets in the event"
    )]
    InvalidRemainingAccounts,
    /// 17 - Max tokens minted
    #[error("This event has completed")]
    BurnEventCompleted,
    /// 18 - Event ended
    #[error("This event has ended")]
    EventEnded,
    /// 19 - Event inactive
    #[error("This event is inactive")]
    EventInactive,
    /// 20 - Invalid max burns
    #[error("Max burns must be greater than 0")]
    InvalidMaxBurns,
    /// 21 - Invalid max burns
    #[error("Max burns must be greater than 0")]
    InvalidParams,
}

impl PrintProgramError for SoulburnError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<SoulburnError> for ProgramError {
    fn from(e: SoulburnError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SoulburnError {
    fn type_of() -> &'static str {
        "Soulburn Error"
    }
}
