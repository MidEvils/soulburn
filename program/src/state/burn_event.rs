use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::SoulburnError, state::Key};

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug)]
pub enum EventType {
    Token { tokens_per_burn: u64, mint: Pubkey },
    Noop,
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug)]
pub enum EndType {
    MaxBurns { max_burns: u16 },
    Timestamp { ends_at: i64 },
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct BurnEvent {
    pub key: Key,
    pub burner: Pubkey,
    pub active: bool,
    pub burns_required: u8,
    pub event_type: EventType,
    pub end_type: Option<EndType>,
    pub burns_completed: u16,
    pub completed: bool,
}

impl BurnEvent {
    pub const LEN: usize = 1 + 32 + 1 + 1 + (1 + 8 + 32) + (1 + (1 + 8)) + 2 + 1;

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
