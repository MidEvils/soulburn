use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use spl_token::instruction as token_instruction;
use spl_token::state::{Account as SplAccount, Mint as SplMint};

use crate::error::SoulburnError;
use crate::state::burn_event::BurnEvent;

pub(crate) fn mint_tokens<'a>(
    mint: &AccountInfo<'a>,
    ata: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    burn_event: &AccountInfo<'a>,
    burner: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    seeds: &Vec<&[u8]>,
    amount: u64,
) -> ProgramResult {
    let token_account = SplAccount::unpack(&ata.try_borrow_data()?)?;
    if token_account.mint != *mint.key || token_account.owner != *owner.key {
        return Err(SoulburnError::AccountMismatch.into());
    }

    let burn_event_account = BurnEvent::load(burn_event)?;
    let mint_account = SplMint::unpack(&mint.try_borrow_data()?)?;

    match burn_event_account.max_tokens {
        Some(tokens) => {
            if mint_account.supply.checked_add(amount).unwrap() > tokens {
                return Err(SoulburnError::MaxTokensMinted.into());
            }
        }
        None => {}
    }

    let decimals = mint_account.decimals;
    let amount_base = amount
        .checked_mul(10u64.pow(decimals as u32))
        .ok_or(SoulburnError::NumericalOverflow)?;

    let ix = token_instruction::mint_to(
        token_program.key,
        mint.key,
        ata.key,
        burner.key,
        &[&burner.key],
        amount_base,
    )?;

    invoke_signed(
        &ix,
        &[mint.clone(), ata.clone(), owner.clone(), burner.clone()],
        &[seeds],
    )
}
