use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, sysvar::Sysvar,
};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys, assert_signer, assert_writable},
    error::SoulburnError,
    instruction::accounts::CreateBurnEventAccounts,
    state::{burn_event::BurnEvent, burner::Burner, mint::Mint, Key},
    utils::{create_account, create_and_init_mint::create_and_init_mint},
};

pub(crate) fn create_burn_event<'a>(
    accounts: &'a [AccountInfo<'a>],
    burns_required: u8,
    tokens_per_event_burn: u64,
    ends_at: Option<i64>,
    max_tokens: Option<u64>,
) -> ProgramResult {
    let ctx = CreateBurnEventAccounts::context(accounts)?;

    assert_signer("authority", ctx.accounts.authority)?;
    assert_signer("burn_event", ctx.accounts.burn_event)?;
    assert_writable("burn_event", ctx.accounts.burn_event)?;

    let burner = Burner::load(ctx.accounts.burner)?;

    assert_same_pubkeys("authority", ctx.accounts.authority, &burner.authority)?;

    let bump = assert_pda(
        "mint",
        ctx.accounts.mint,
        &crate::ID,
        &Mint::seeds(ctx.accounts.burn_event.key),
    )?;

    match ends_at {
        Some(time) => {
            let now = Clock::get().unwrap().unix_timestamp;
            if time <= now {
                msg!("Invalid end time");
                return Err(SoulburnError::InvalidEndTime.into());
            }
        }
        None => (),
    }

    // Create Burn Event.
    let burn_event = BurnEvent {
        key: Key::BurnEvent,
        ends_at,
        max_tokens,
        tokens_per_event_burn,
        mint: *ctx.accounts.mint.key,
        active: false,
        burner: *ctx.accounts.burner.key,
        burns_required,
    };

    create_account(
        ctx.accounts.burn_event,
        ctx.accounts.authority,
        ctx.accounts.system_program,
        BurnEvent::LEN,
        &crate::ID,
        None,
    )?;

    let mut mint_seeds = Mint::seeds(ctx.accounts.burn_event.key);
    let mint_seeds_bump = &[bump];
    mint_seeds.push(mint_seeds_bump);

    create_and_init_mint(
        ctx.accounts.authority,
        ctx.accounts.mint,
        ctx.accounts.system_program,
        ctx.accounts.token_program,
        ctx.accounts.burner,
        &mint_seeds,
        0,
    )?;

    burn_event.save(ctx.accounts.burn_event)
}
