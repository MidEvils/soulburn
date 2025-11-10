use mpl_core::instructions::{BurnV1CpiBuilder, CreateV1CpiBuilder};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    assertions::{assert_mpl_core_asset, assert_pda},
    state::{asset::Asset, burner::Burner},
};

pub(crate) fn soulburn<'a>(
    asset_info: &AccountInfo<'a>,
    collection_info: &AccountInfo<'a>,
    soulbound_asset_info: &AccountInfo<'a>,
    soulbound_collection_info: &AccountInfo<'a>,
    burner_info: &AccountInfo<'a>,
    owner_info: &AccountInfo<'a>,
    core_program_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let burner_bump = assert_pda(
        "burner",
        burner_info,
        &crate::ID,
        &Burner::seeds(collection_info.key),
    )?;

    let asset_bump = assert_pda(
        "soulbound_asset",
        soulbound_asset_info,
        &crate::ID,
        &Asset::seeds(collection_info.key, asset_info.key),
    )?;

    let asset = assert_mpl_core_asset("asset", asset_info, collection_info.key)?;

    let mut burner_seeds = Burner::seeds(collection_info.key);
    let burner_seeds_bump = &[burner_bump];
    burner_seeds.push(burner_seeds_bump);

    let mut asset_seeds = Asset::seeds(collection_info.key, asset_info.key);
    let asset_seeds_bump = &[asset_bump];
    asset_seeds.push(asset_seeds_bump);

    CreateV1CpiBuilder::new(core_program_info)
        .asset(soulbound_asset_info)
        .name(asset.base.name)
        .uri(asset.base.uri)
        .owner(Some(owner_info))
        .collection(Some(soulbound_collection_info))
        .authority(Some(burner_info))
        .payer(owner_info)
        .system_program(system_program_info)
        .invoke_signed(&[&burner_seeds, &asset_seeds])?;

    BurnV1CpiBuilder::new(core_program_info)
        .asset(asset_info)
        .collection(Some(collection_info))
        .authority(Some(owner_info))
        .payer(owner_info)
        .invoke()
}
