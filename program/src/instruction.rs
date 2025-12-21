use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::state::burn_event::EndType;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum SoulburnInstruction {
    /// Creates the burner account derived from the provided authority.
    #[account(0, writable, name="burner", desc = "The PDA of the burner account to create (seeds: ['soulburn', 'burner', collection])")]
    #[account(1, signer, name="authority", desc = "The authority of the burner")]
    #[account(2, name="collection", desc = "The collection for the burner")]
    #[account(3, writable, name="soulbound_collection", desc = "The PDA of the collection to create (seeds: ['soulburn', 'collection', collection])")]
    #[account(4, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(5, name="core_program", desc = "The mpl-core program")]
    #[account(6, name="system_program", desc = "The system program")]
    Create { name: String, uri: String },

    /// Soulburns an asset using a burner, not linked to any event rewards.
    #[account(0, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(1, writable, name="asset", desc = "The asset to be burned")]
    #[account(2, writable, name="collection", desc = "The collection of the asset to be burned")]
    #[account(3, writable, name="soulbound_collection", desc = "The collection of the asset to be burned")]
    #[account(4, writable, name="soulbound_asset", desc = "The collection of the asset to be burned (seeds: ['soulburn', 'asset', collection, asset])")]
    #[account(5, writable, signer, name="owner", desc = "The owner of the asset")]
    #[account(6, name="core_program", desc = "The mpl-core program")]
    #[account(7, name="system_program", desc = "The system program")]
    SoulburnAsset,

    /// Creates burn event where assets are burned for spl tokens (eg mint token for CM)
    #[account(0, writable, signer, name="burn_event", desc = "The account of the burn-event")]
    #[account(1, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(2, signer, name="authority", desc = "The authority of the burner")]
    #[account(3, writable, optional, name="mint", desc = "The mint of the spl-token (seeds: ['soulburn', 'mint', burn_event])")]
    #[account(4, name="token_program", desc = "The spl-token program")]
    #[account(5, name="system_program", desc = "The system program")]
    CreateBurnEvent { burns_required: u8, end_type: Option<EndType>, tokens_per_burn: Option<u64>  },

    /// Toggles event active
    #[account(0, writable, name="burn_event", desc = "The account of the burn-event")]
    #[account(1, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(2, signer, name="authority", desc = "The authority of the burner")]
    ToggleEventActive { active: bool },

    /// Deletes burn event
    #[account(0, writable, name="burn_event", desc = "The account of the burn-event")]
    #[account(1, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(2, signer, writable, name="authority", desc = "The authority of the burner")]
    DeleteBurnEvent,

    /// Burn asset(s) as part of a burn event.
    /// Additional accounts to be provided in pairs of [Asset, SoulboundAsset]
    #[account(0, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(1, writable, name="burn_event", desc = "The burn event account")]
    #[account(2, writable, name="collection", desc = "The collection of the asset to be burned")]
    #[account(3, writable, name="soulbound_collection", desc = "The collection of the soulbound asset")]
    #[account(4, writable, signer, name="owner", desc = "The owner of the asset")]
    #[account(5, optional, writable, name="mint", desc = "The mint of the event (seeds: ['soulburn', 'mint', burn_event])")]
    #[account(6, optional, writable, name="ata", desc = "The ata to receive the reward mint")]
    #[account(7, name="core_program", desc = "The mpl-core program")]
    #[account(8, name="system_program", desc = "The system program")]
    #[account(9, name="token_program", desc = "The token program")]
    #[account(10, optional, name="associated_token_program", desc = "The associated token program")]
    EventBurn,

    /// Amin instruction to manually mint a burn event token
    #[account(0, name="burner", desc = "The PDA of the burner account (seeds: ['soulburn', 'burner', collection])")]
    #[account(1, writable, name="burn_event", desc = "The burn event account")]
    #[account(2, writable, name="mint", desc = "The mint of the event (seeds: ['soulburn', 'mint', burn_event])")]
    #[account(3, signer, name="authority", desc = "The authority of the burner")]
    #[account(4, writable, name="ata", desc = "The ata to receive the reward mint")]
    #[account(5, name="owner", desc = "The owner of the ata")]
    #[account(6, name="system_program", desc = "The system program")]
    #[account(7, name="token_program", desc = "The token program")]
    #[account(8, name="associated_token_program", desc = "The associated token program")]
    AdminMint { amount: u16 }
}
