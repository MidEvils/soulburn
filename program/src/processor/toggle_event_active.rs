use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    assertions::{assert_same_pubkeys, assert_signer},
    instruction::accounts::ToggleEventActiveAccounts,
    state::{burn_event::BurnEvent, burner::Burner},
};

pub(crate) fn toggle_event_active<'a>(
    accounts: &'a [AccountInfo<'a>],
    active: bool,
) -> ProgramResult {
    let ctx = ToggleEventActiveAccounts::context(accounts)?;
    let burner = Burner::load(ctx.accounts.burner)?;

    assert_same_pubkeys("authority", ctx.accounts.authority, &burner.authority)?;
    assert_signer("authority", ctx.accounts.authority)?;

    let mut burn_event = BurnEvent::load(ctx.accounts.burn_event)?;

    assert_same_pubkeys("burner", ctx.accounts.burner, &burn_event.burner)?;

    burn_event.active = active;

    burn_event.save(ctx.accounts.burn_event)
}
