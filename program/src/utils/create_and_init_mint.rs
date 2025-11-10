use solana_program::system_instruction;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, program::invoke_signed,
    program_pack::Pack, rent::Rent, sysvar::Sysvar,
};
use spl_token::id as spl_token_id;
use spl_token::instruction as token_instruction;
use spl_token::state::Mint as SplMint;

pub(crate) fn create_and_init_mint<'a>(
    payer: &AccountInfo<'a>,
    token_mint: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    mint_authority: &AccountInfo<'a>,
    mint_seeds: &Vec<&[u8]>,
    token_decimals: u8,
) -> ProgramResult {
    let space = SplMint::LEN as u64;
    let lamports = Rent::get()?.minimum_balance(space as usize);

    invoke_signed(
        &system_instruction::create_account(
            &payer.key,
            &token_mint.key,
            lamports,
            space,
            &token_program.key,
        ),
        &[payer.clone(), token_mint.clone(), system_program.clone()],
        &[mint_seeds],
    )?;

    let initialize_ix = token_instruction::initialize_mint2(
        &spl_token_id(),
        token_mint.key,
        mint_authority.key,
        None,
        token_decimals,
    )?;

    invoke(&initialize_ix, &[token_mint.clone()])
}
