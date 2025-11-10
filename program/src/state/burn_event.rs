use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::SoulburnError, state::Key};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct BurnEvent {
    pub key: Key,
    pub burner: Pubkey,
    pub mint: Pubkey,
    pub active: bool,
    pub ends_at: Option<i64>,
    pub burns_required: u8,
    pub tokens_per_event_burn: u64,
    pub max_tokens: Option<u64>,
}

impl BurnEvent {
    pub const LEN: usize = 1 + 32 + 32 + 1 + (1 + 8) + 1 + 8 + (1 + 8);

    pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow();
        BurnEvent::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            SoulburnError::DeserializationError.into()
        })
    }

    pub fn save(&self, account: &AccountInfo) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[..], self).map_err(|error| {
            msg!("Error: {}", error);
            SoulburnError::SerializationError.into()
        })
    }
}
