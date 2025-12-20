use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use borsh::BorshDeserialize;

use crate::instruction::SoulburnInstruction;

mod admin_mint;
mod create;
mod create_burn_event;
mod event_burn;
mod soulburn_asset;
mod toggle_event_active;
use admin_mint::*;
use create::*;
use create_burn_event::*;
use event_burn::*;
use soulburn_asset::*;
use toggle_event_active::*;

pub fn process_instruction<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: SoulburnInstruction = SoulburnInstruction::try_from_slice(instruction_data)?;
    match instruction {
        SoulburnInstruction::Create { name, uri } => {
            msg!("Instruction: Create");
            create(accounts, name, uri)
        }
        SoulburnInstruction::SoulburnAsset => {
            msg!("Instruction: Soulburn");
            soulburn_asset(accounts)
        }
        SoulburnInstruction::CreateBurnEvent {
            burns_required,
            end_type,
            tokens_per_burn,
        } => {
            msg!("Instruction: CreateBurnEvent");
            create_burn_event(accounts, burns_required, end_type, tokens_per_burn)
        }
        SoulburnInstruction::EventBurn => {
            msg!("Instruction: EventBurn");
            event_burn(accounts)
        }
        SoulburnInstruction::AdminMint { amount } => {
            msg!("Instruction: AdminMint");
            admin_mint(accounts, amount)
        }
        SoulburnInstruction::ToggleEventActive { active } => {
            msg!("Instruction: ToggleEventActive");
            toggle_event_active(accounts, active)
        }
    }
}
