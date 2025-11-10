use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::{FromPrimitive, ToPrimitive};
use strum::EnumIter;

pub mod asset;
pub mod burn_event;
pub mod burner;
pub mod collection;
pub mod mint;
mod traits;
pub use traits::*;

pub(crate) const PREFIX: &str = "soulburn";

#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    Debug,
    PartialEq,
    Eq,
    ToPrimitive,
    FromPrimitive,
    EnumIter,
)]
pub enum Key {
    Uninitialized,
    Burner,
    BurnEvent,
    Collection,
    Asset,
    Mint,
}

impl Key {
    const BASE_LEN: usize = 1; // 1 byte for the discriminator
}

impl DataBlob for Key {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}
