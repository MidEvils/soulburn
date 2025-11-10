use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::state::{Key, PREFIX};

#[derive(ShankAccount)]
pub struct Collection {
    _key: Key,
}

impl Collection {
    pub fn seeds(collection: &Pubkey) -> Vec<&[u8]> {
        vec![
            PREFIX.as_bytes(),
            "collection".as_bytes(),
            collection.as_ref(),
        ]
    }
}
