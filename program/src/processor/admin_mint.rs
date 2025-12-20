use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys, assert_signer},
    error::SoulburnError,
    instruction::accounts::AdminMintAccounts,
    state::{
        burn_event::{BurnEvent, EndType},
        burner::Burner,
    },
    utils::{create_ata::create_ata, mint_tokens::mint_tokens},
};

pub(crate) fn admin_mint<'a>(accounts: &'a [AccountInfo<'a>], amount: u16) -> ProgramResult {
    let ctx = AdminMintAccounts::context(accounts)?;

    let burner = Burner::load(ctx.accounts.burner)?;
    let mut burn_event = BurnEvent::load(ctx.accounts.burn_event)?;

    let bump = assert_pda(
        "burner",
        ctx.accounts.burner,
        &crate::ID,
        &Burner::seeds(&burner.collection),
    )?;

    match burn_event.end_type {
        Some(end_type) => match end_type {
            EndType::Timestamp { ends_at: _ } => {}
            EndType::MaxBurns { max_burns } => {
                if burn_event.burns_completed + amount > max_burns {
                    msg!("Burn event completed");
                    return Err(SoulburnError::BurnEventCompleted.into());
                }
            }
        },
        None => (),
    }

    assert_same_pubkeys("burner", ctx.accounts.burner, &burn_event.burner)?;
    assert_same_pubkeys("authority", ctx.accounts.authority, &burner.authority)?;
    assert_signer("authority", ctx.accounts.authority)?;

    create_ata(
        ctx.accounts.authority,
        ctx.accounts.owner,
        ctx.accounts.ata,
        ctx.accounts.mint,
        ctx.accounts.token_program,
        ctx.accounts.system_program,
        ctx.accounts.associated_token_program,
    )?;

    let mut seeds = Burner::seeds(&burner.collection);
    let bump_seeds = &[bump];
    seeds.push(bump_seeds);

    mint_tokens(
        ctx.accounts.mint,
        ctx.accounts.ata,
        ctx.accounts.owner,
        ctx.accounts.burn_event,
        ctx.accounts.burner,
        ctx.accounts.token_program,
        &seeds,
        u64::from(amount),
    )?;

    burn_event.burns_completed += amount;
    burn_event.save(ctx.accounts.burn_event)
}
