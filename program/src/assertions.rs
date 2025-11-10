use crate::{error::SoulburnError, state::Key};
use mpl_core::{
    types::{Key as MplCoreKey, UpdateAuthority},
    Asset, ID as MPL_CORE_ID,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Assert that the given account is owned by the given program or one of the given owners.
/// Useful for dealing with program interfaces.
pub fn assert_program_owner_either(
    account_name: &str,
    account: &AccountInfo,
    owners: &[Pubkey],
) -> ProgramResult {
    if !owners.iter().any(|owner| account.owner == owner) {
        msg!(
            "Account \"{}\" [{}] must be owned by either {:?}",
            account_name,
            account.key,
            owners
        );
        Err(SoulburnError::InvalidProgramOwner.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is owned by the given program.
pub fn assert_program_owner(
    account_name: &str,
    account: &AccountInfo,
    owner: &Pubkey,
) -> ProgramResult {
    if account.owner != owner {
        msg!(
            "Account \"{}\" [{}] expected program owner [{}], got [{}]",
            account_name,
            account.key,
            owner,
            account.owner
        );
        Err(SoulburnError::InvalidProgramOwner.into())
    } else {
        Ok(())
    }
}

/// Assert the derivation of the seeds against the given account and return the bump seed.
pub fn assert_pda(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds: &[&[u8]],
) -> Result<u8, ProgramError> {
    let (key, bump) = Pubkey::find_program_address(seeds, program_id);
    if *account.key != key {
        msg!(
            "Account \"{}\" [{}] is an invalid PDA. Expected the following valid PDA [{}]",
            account_name,
            account.key,
            key,
        );
        return Err(SoulburnError::InvalidPda.into());
    }
    Ok(bump)
}

/// Assert the derivation of the seeds plus bump against the given account.
pub fn assert_pda_with_bump(
    account_name: &str,
    account: &AccountInfo,
    program_id: &Pubkey,
    seeds_with_bump: &[&[u8]],
) -> ProgramResult {
    let key = Pubkey::create_program_address(seeds_with_bump, program_id)?;
    if *account.key != key {
        msg!(
            "Account \"{}\" [{}] is an invalid PDA. Expected the following valid PDA [{}]",
            account_name,
            account.key,
            key,
        );
        Err(SoulburnError::InvalidPda.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is empty.
pub fn assert_empty(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.data_is_empty() {
        msg!(
            "Account \"{}\" [{}] must be empty",
            account_name,
            account.key,
        );
        Err(SoulburnError::ExpectedEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is non empty.
pub fn assert_non_empty(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if account.data_is_empty() {
        msg!(
            "Account \"{}\" [{}] must not be empty",
            account_name,
            account.key,
        );
        Err(SoulburnError::ExpectedNonEmptyAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is a signer.
pub fn assert_signer(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_signer {
        msg!(
            "Account \"{}\" [{}] must be a signer",
            account_name,
            account.key,
        );
        Err(SoulburnError::ExpectedSignerAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is writable.
pub fn assert_writable(account_name: &str, account: &AccountInfo) -> ProgramResult {
    if !account.is_writable {
        msg!(
            "Account \"{}\" [{}] must be writable",
            account_name,
            account.key,
        );
        Err(SoulburnError::ExpectedWritableAccount.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account matches the given public key.
pub fn assert_same_pubkeys(
    account_name: &str,
    account: &AccountInfo,
    expected: &Pubkey,
) -> ProgramResult {
    if account.key != expected {
        msg!(
            "Account \"{}\" [{}] must match the following public key [{}]",
            account_name,
            account.key,
            expected
        );
        Err(SoulburnError::AccountMismatch.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account has the expected account key.
pub fn assert_account_key(account_name: &str, account: &AccountInfo, key: Key) -> ProgramResult {
    let key_number = key as u8;
    if account.data_len() <= 1 || account.try_borrow_data()?[0] != key_number {
        msg!(
            "Account \"{}\" [{}] expected account key [{}], got [{}]",
            account_name,
            account.key,
            key_number,
            account.try_borrow_data()?[0]
        );
        Err(SoulburnError::InvalidAccountKey.into())
    } else {
        Ok(())
    }
}

/// Assert that the given account is an mpl-core asset and a member of given collection
pub fn assert_mpl_core_asset(
    account_name: &str,
    account: &AccountInfo,
    collection: &Pubkey,
) -> Result<Box<Asset>, ProgramError> {
    assert_program_owner(account_name, account, &MPL_CORE_ID)?;
    let asset = Asset::from_bytes(&**account.data.borrow())
        .map_err(|_| SoulburnError::ExpectedMplCoreAsset)?;
    if asset.base.key != MplCoreKey::AssetV1 {
        msg!(
            "Account \"{}\" [{}] expected [{:?}], got [{:?}]",
            account_name,
            account.key,
            MplCoreKey::AssetV1,
            asset.base.key
        );
        Err(SoulburnError::ExpectedMplCoreAsset.into())
    } else if asset.base.update_authority != UpdateAuthority::Collection(*collection) {
        msg!(
            "Account \"{}\" [{}] invalid collection, expected [{:?}], got [{:?}]",
            account_name,
            account.key,
            collection,
            asset.base.update_authority
        );
        Err(SoulburnError::InvalidCollectionForAsset.into())
    } else {
        Ok(asset)
    }
}
