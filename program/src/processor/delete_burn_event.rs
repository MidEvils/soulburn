use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    assertions::{assert_same_pubkeys, assert_signer},
    instruction::accounts::ToggleEventActiveAccounts,
    state::{burn_event::BurnEvent, burner::Burner},
    utils::close_account,
};

pub(crate) fn delete_burn_event<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = ToggleEventActiveAccounts::context(accounts)?;
    let burner = Burner::load(ctx.accounts.burner)?;

    assert_same_pubkeys("authority", ctx.accounts.authority, &burner.authority)?;
    assert_signer("authority", ctx.accounts.authority)?;

    let burn_event = BurnEvent::load(ctx.accounts.burn_event)?;

    assert_same_pubkeys("burner", ctx.accounts.burner, &burn_event.burner)?;

    close_account(ctx.accounts.burn_event, ctx.accounts.authority)
}
