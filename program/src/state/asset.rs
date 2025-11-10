use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::state::{Key, PREFIX};

#[derive(ShankAccount)]
pub struct Asset {
    _key: Key,
}

impl Asset {
    pub fn seeds<'a>(collection: &'a Pubkey, asset: &'a Pubkey) -> Vec<&'a [u8]> {
        vec![
            PREFIX.as_bytes(),
            "asset".as_bytes(),
            collection.as_ref(),
            asset.as_ref(),
        ]
    }
}
