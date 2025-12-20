use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, system_program,
    sysvar::Sysvar,
};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys},
    error::SoulburnError,
    instruction::accounts::EventBurnAccounts,
    state::{
        burn_event::{BurnEvent, EndType, EventType},
        burner::Burner,
    },
    utils::{create_ata::create_ata, mint_tokens::mint_tokens, soulburn::soulburn},
};

use spl_token::id as spl_token_id;

pub(crate) fn event_burn<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = EventBurnAccounts::context(accounts)?;

    let mut burn_event = BurnEvent::load(ctx.accounts.burn_event)?;

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

    match burn_event.end_type {
        Some(end_type) => match end_type {
            EndType::Timestamp { ends_at } => {
                let now = Clock::get().unwrap().unix_timestamp;
                if now >= ends_at {
                    msg!("Event ended");
                    return Err(SoulburnError::EventEnded.into());
                }
            }
            EndType::MaxBurns { max_burns } => {
                if burn_event.burns_completed >= max_burns {
                    msg!("Burn event completed");
                    return Err(SoulburnError::BurnEventCompleted.into());
                }
            }
        },
        None => (),
    }

    if burn_event.burns_required != u8::try_from(ctx.remaining_accounts.len() / 2).unwrap() {
        msg!("Invalid remaining accounts");
        return Err(SoulburnError::InvalidRemainingAccounts.into());
    }

    for chunk in ctx.remaining_accounts.chunks(2) {
        soulburn(
            &chunk[0],
            ctx.accounts.collection,
            &chunk[1],
            ctx.accounts.soulbound_collection,
            ctx.accounts.burner,
            ctx.accounts.owner,
            ctx.accounts.core_program,
            ctx.accounts.system_program,
        )?
    }

    match burn_event.event_type {
        EventType::Token {
            tokens_per_burn: amount,
            mint: _,
        } => {
            create_ata(
                ctx.accounts.owner,
                ctx.accounts.owner,
                ctx.accounts.ata.unwrap(),
                ctx.accounts.mint.unwrap(),
                ctx.accounts.token_program,
                ctx.accounts.system_program,
                ctx.accounts.associated_token_program.unwrap(),
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
            mint_tokens(
                ctx.accounts.mint.unwrap(),
                ctx.accounts.ata.unwrap(),
                ctx.accounts.owner,
                ctx.accounts.burn_event,
                ctx.accounts.burner,
                ctx.accounts.token_program,
                &seeds,
                amount,
            )?;
        }
        EventType::Noop => (),
    }

    burn_event.burns_completed += 1;
    burn_event.save(ctx.accounts.burn_event)
}
