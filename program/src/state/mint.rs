use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::state::{Key, PREFIX};

#[derive(ShankAccount)]
pub struct Mint {
    _key: Key,
}

impl Mint {
    pub fn seeds(burn_event: &Pubkey) -> Vec<&[u8]> {
        vec![PREFIX.as_bytes(), "mint".as_bytes(), burn_event.as_ref()]
    }
}
