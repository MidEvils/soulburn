use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::SoulburnError;
use crate::state::{Key, SolanaAccount, PREFIX};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct Burner {
    pub key: Key,
    pub authority: Pubkey,
    pub collection: Pubkey,
}

impl Burner {
    pub const LEN: usize = 1 + 32 + 32;

    pub fn seeds(collection: &Pubkey) -> Vec<&[u8]> {
        vec![PREFIX.as_bytes(), "burner".as_bytes(), collection.as_ref()]
    }

    pub fn find_pda(collection: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&Self::seeds(collection), &crate::ID)
    }

    pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Burner::deserialize(&mut bytes).map_err(|error| {
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

impl SolanaAccount for Burner {
    fn key() -> Key {
        Key::Burner
    }
}
