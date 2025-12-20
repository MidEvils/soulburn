use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, sysvar::Sysvar,
};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys, assert_signer, assert_writable},
    error::SoulburnError,
    instruction::accounts::CreateBurnEventAccounts,
    state::{
        burn_event::{BurnEvent, EndType, EventType},
        burner::Burner,
        mint::Mint,
        Key,
    },
    utils::{create_account, create_and_init_mint::create_and_init_mint},
};

pub(crate) fn create_burn_event<'a>(
    accounts: &'a [AccountInfo<'a>],
    burns_required: u8,
    end_type: Option<EndType>,
    tokens_per_burn: Option<u64>,
) -> ProgramResult {
    let ctx = CreateBurnEventAccounts::context(accounts)?;

    assert_signer("authority", ctx.accounts.authority)?;
    assert_signer("burn_event", ctx.accounts.burn_event)?;
    assert_writable("burn_event", ctx.accounts.burn_event)?;

    let burner = Burner::load(ctx.accounts.burner)?;

    assert_same_pubkeys("authority", ctx.accounts.authority, &burner.authority)?;

    let event_type = match ctx.accounts.mint {
        Some(acc) => match tokens_per_burn {
            Some(tokens_per_burn) => EventType::Token {
                tokens_per_burn,
                mint: *acc.key,
            },
            None => {
                msg!("Invalid params");
                return Err(SoulburnError::InvalidParams.into());
            }
        },
        None => EventType::Noop,
    };

    match event_type {
        EventType::Token {
            tokens_per_burn: _,
            mint: _,
        } => {
            let bump = assert_pda(
                "mint",
                ctx.accounts.mint.unwrap(),
                &crate::ID,
                &Mint::seeds(ctx.accounts.burn_event.key),
            )?;
            let mut mint_seeds = Mint::seeds(ctx.accounts.burn_event.key);
            let mint_seeds_bump = &[bump];
            mint_seeds.push(mint_seeds_bump);

            create_and_init_mint(
                ctx.accounts.authority,
                ctx.accounts.mint.unwrap(),
                ctx.accounts.system_program,
                ctx.accounts.token_program,
                ctx.accounts.burner,
                &mint_seeds,
                0,
            )?;
        }
        EventType::Noop => (),
    }

    match end_type {
        Some(type_of_end) => match type_of_end {
            EndType::Timestamp { ends_at } => {
                let now = Clock::get().unwrap().unix_timestamp;
                if ends_at <= now {
                    msg!("Invalid end time");
                    return Err(SoulburnError::InvalidEndTime.into());
                }
            }
            EndType::MaxBurns { max_burns } => {
                if max_burns <= 0u16 {
                    msg!("Invalid max burns");
                    return Err(SoulburnError::InvalidMaxBurns.into());
                }
            }
        },
        None => (),
    }

    // Create Burn Event.
    let burn_event = BurnEvent {
        key: Key::BurnEvent,
        burner: *ctx.accounts.burner.key,
        active: false,
        burns_required,
        event_type,
        end_type,
        burns_completed: 0,
        completed: false,
    };

    create_account(
        ctx.accounts.burn_event,
        ctx.accounts.authority,
        ctx.accounts.system_program,
        BurnEvent::LEN,
        &crate::ID,
        None,
    )?;

    burn_event.save(ctx.accounts.burn_event)
}
