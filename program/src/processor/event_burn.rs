use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, system_program,
    sysvar::Sysvar,
};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys},
    error::SoulburnError,
    instruction::accounts::EventBurnAccounts,
    state::{burn_event::BurnEvent, burner::Burner},
    utils::{create_ata::create_ata, mint_tokens::mint_tokens, soulburn::soulburn},
};

use spl_token::id as spl_token_id;

pub(crate) fn event_burn<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = EventBurnAccounts::context(accounts)?;

    let burn_event = BurnEvent::load(ctx.accounts.burn_event)?;

    if !burn_event.active {
        msg!("Event inactive");
        return Err(SoulburnError::EventInactive.into());
    }
    assert_same_pubkeys("burner", ctx.accounts.burner, &burn_event.burner)?;

    assert_same_pubkeys(
        "system_program",
        ctx.accounts.system_program,
        &system_program::id(),
    )?;

    assert_same_pubkeys("token_program", ctx.accounts.token_program, &spl_token_id())?;

    match burn_event.ends_at {
        Some(ends_at) => {
            let now = Clock::get().unwrap().unix_timestamp;
            if now >= ends_at {
                msg!("Event ended");
                return Err(SoulburnError::EventEnded.into());
            }
        }
        None => {}
    }

    if burn_event.burns_required != u8::try_from(ctx.remaining_accounts.len() / 2).unwrap() {
        msg!("Invalid remaining accounts");
        return Err(SoulburnError::InvalidRemainingAccounts.into());
    }

    ctx.remaining_accounts.chunks(2).for_each(|chunk| {
        let _ = soulburn(
            &chunk[0],
            ctx.accounts.collection,
            &chunk[1],
            ctx.accounts.soulbound_collection,
            ctx.accounts.burner,
            ctx.accounts.owner,
            ctx.accounts.core_program,
            ctx.accounts.system_program,
        );
    });

    create_ata(
        ctx.accounts.owner,
        ctx.accounts.owner,
        ctx.accounts.ata,
        ctx.accounts.mint,
        ctx.accounts.token_program,
        ctx.accounts.system_program,
        ctx.accounts.associated_token_program,
    )?;

    let bump = assert_pda(
        "burner",
        ctx.accounts.burner,
        &crate::ID,
        &Burner::seeds(ctx.accounts.collection.key),
    )?;

    let mut seeds = Burner::seeds(ctx.accounts.collection.key);
    let seeds_bump = &[bump];
    seeds.push(seeds_bump);

    let amount = burn_event.tokens_per_event_burn;

    mint_tokens(
        ctx.accounts.mint,
        ctx.accounts.ata,
        ctx.accounts.owner,
        ctx.accounts.burn_event,
        ctx.accounts.burner,
        ctx.accounts.token_program,
        &seeds,
        amount,
    )
}
