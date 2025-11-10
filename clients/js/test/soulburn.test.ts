import test from 'ava';
import {
  findAssetPda,
  findCollectionPda,
  getSoulburnAssetInstruction,
} from '../src';
import {
  createBurner,
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreAsset, createCoreCollection } from './_mpl-core';
import {
  AssetV1,
  fetchAssetV1,
  getTransferV1Instruction,
  MPL_CORE_PROGRAM_ERROR__INVALID_AUTHORITY,
  MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
} from '../sdks/mpl-core/generated';
import {
  Account,
  appendTransactionMessageInstruction,
  fetchEncodedAccount,
  isProgramError,
  isSolanaError,
  pipe,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
} from '@solana/kit';

test('can soulburn an asset', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);

  const owner = await generateKeyPairSignerWithSol(client);

  const name = 'Test asset';
  const uri = 'https://example.com';

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    name,
    uri
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  // When we create a new burner account.
  const soulburnIx = getSoulburnAssetInstruction({
    burner,
    collection,
    soulboundCollection,
    asset,
    soulboundAsset,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const assetAccount = await fetchEncodedAccount(client.rpc, asset);
  // asset isn't completely burned, just set to uninitialized
  t.assert(assetAccount.exists && assetAccount.space === 1n);

  const soulboundAssetAccount = await fetchAssetV1(client.rpc, soulboundAsset);

  t.like(soulboundAssetAccount, <Account<AssetV1>>{
    data: {
      owner: owner.address,
      name,
      uri,
      updateAuthority: {
        __kind: 'Collection',
      },
    },
  });

  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getTransferV1Instruction({
          asset: soulboundAsset,
          collection: soulboundCollection,
          payer: owner,
          newOwner: authority.address,
          compressionProof: null,
        }),
        tx
      )
  );

  const promise = signAndSendTransaction(client, transactionMessage);
  const error = await t.throwsAsync(promise);
  t.true(
    isSolanaError(
      error,
      SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE
    )
  );
  t.true(
    isProgramError(
      error.cause,
      transactionMessage,
      MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
      MPL_CORE_PROGRAM_ERROR__INVALID_AUTHORITY
    )
  );
});
