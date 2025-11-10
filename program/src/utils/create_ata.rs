use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, program::invoke};
use spl_associated_token_account;
use spl_token::id as spl_token_id;

use crate::error::SoulburnError;

pub(crate) fn create_ata<'a>(
    payer: &AccountInfo<'a>,
    owner: &AccountInfo<'a>,
    ata: &AccountInfo<'a>,
    token_mint: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    associated_token_program: &AccountInfo<'a>,
) -> ProgramResult {
    let expected_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        owner.key,
        token_mint.key,
        &spl_token_id(),
    );

    if ata.key != &expected_ata {
        return Err(SoulburnError::InvalidPda.into());
    }

    // if acc exists, noop.
    if !ata.data_is_empty() {
        return Ok(());
    }

    let ix = spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        payer.key,
        owner.key,
        token_mint.key,
        &spl_token_id(),
    );

    invoke(
        &ix,
        &[
            payer.clone(),
            ata.clone(),
            owner.clone(),
            token_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )
}
