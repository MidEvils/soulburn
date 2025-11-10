use mpl_core::{
    instructions::CreateCollectionV1CpiBuilder,
    types::{PermanentFreezeDelegate, PluginAuthorityPair},
    ID as MPL_CORE_ID,
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, system_program};

use crate::{
    assertions::{assert_pda, assert_same_pubkeys, assert_signer, assert_writable},
    instruction::accounts::CreateAccounts,
    state::{burner::Burner, collection::Collection, Key},
    utils::create_account,
};

pub(crate) fn create<'a>(
    accounts: &'a [AccountInfo<'a>],
    name: String,
    uri: String,
) -> ProgramResult {
    // Accounts.
    let ctx = CreateAccounts::context(accounts)?;

    // Guards.
    let burner_bump = assert_pda(
        "burner",
        ctx.accounts.burner,
        &crate::ID,
        &Burner::seeds(ctx.accounts.collection.key),
    )?;
    assert_signer("authority", ctx.accounts.authority)?;
    assert_signer("payer", ctx.accounts.payer)?;
    assert_writable("payer", ctx.accounts.payer)?;
    assert_same_pubkeys(
        "system_program",
        ctx.accounts.system_program,
        &system_program::id(),
    )?;

    assert_same_pubkeys("core_program", ctx.accounts.core_program, &MPL_CORE_ID)?;

    // Do nothing if the domain already exists.
    if !ctx.accounts.burner.data_is_empty() {
        return Ok(());
    }

    // Create Burner PDA.
    let burner = Burner {
        key: Key::Burner,
        authority: *ctx.accounts.authority.key,
        collection: *ctx.accounts.collection.key,
    };
    let mut burner_seeds = Burner::seeds(ctx.accounts.collection.key);
    let burner_seeds_bump = &[burner_bump];
    burner_seeds.push(burner_seeds_bump);

    create_account(
        ctx.accounts.burner,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        Burner::LEN,
        &crate::ID,
        Some(&[&burner_seeds]),
    )?;

    let collection_bump = assert_pda(
        "soulbound_collection",
        ctx.accounts.soulbound_collection,
        &crate::ID,
        &Collection::seeds(ctx.accounts.collection.key),
    )?;

    let mut collection_seeds = Collection::seeds(ctx.accounts.collection.key);
    let collection_seeds_bump = &[collection_bump];
    collection_seeds.push(collection_seeds_bump);

    let plugins = vec![PluginAuthorityPair {
        plugin: mpl_core::types::Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
            frozen: true,
        }),
        authority: None,
    }];

    CreateCollectionV1CpiBuilder::new(ctx.accounts.core_program)
        .collection(ctx.accounts.soulbound_collection)
        .name(name)
        .uri(uri)
        .plugins(plugins)
        .payer(ctx.accounts.payer)
        .update_authority(Some(ctx.accounts.burner))
        .system_program(ctx.accounts.system_program)
        .invoke_signed(&[&collection_seeds])?;

    burner.save(ctx.accounts.burner)
}
