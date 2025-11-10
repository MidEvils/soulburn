use crate::{
    assertions::assert_same_pubkeys, instruction::accounts::SoulburnAssetAccounts,
    state::burner::Burner, utils::soulburn::soulburn,
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

pub(crate) fn soulburn_asset<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = SoulburnAssetAccounts::context(accounts)?;

    assert_same_pubkeys(
        "system_program",
        ctx.accounts.system_program,
        &system_program::id(),
    )?;

    let burner = Burner::load(ctx.accounts.burner)?;
    assert_same_pubkeys("collection", ctx.accounts.collection, &burner.collection)?;

    soulburn(
        ctx.accounts.asset,
        ctx.accounts.collection,
        ctx.accounts.soulbound_asset,
        ctx.accounts.soulbound_collection,
        ctx.accounts.burner,
        ctx.accounts.owner,
        ctx.accounts.core_program,
        ctx.accounts.system_program,
    )
}
