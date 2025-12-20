import test from 'ava';
import {
  findAssetPda,
  findCollectionPda,
  findMintPda,
  getEventBurnInstruction,
  SOULBURN_ERROR__EVENT_ENDED,
  SOULBURN_ERROR__EVENT_INACTIVE,
  SOULBURN_ERROR__EXPECTED_MPL_CORE_ASSET,
  SOULBURN_ERROR__INVALID_REMAINING_ACCOUNTS,
  SOULBURN_ERROR__BURN_EVENT_COMPLETED,
  SOULBURN_PROGRAM_ADDRESS,
  endType,
} from '../src';
import {
  activateBurnEvent,
  Client,
  createBurner,
  createBurnEvent,
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreAsset, createCoreCollection } from './_mpl-core';
import {
  AssetV1,
  fetchAllAssetV1,
  MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
} from '../sdks/mpl-core/generated';
import {
  Account,
  Address,
  appendTransactionMessageInstruction,
  appendTransactionMessageInstructions,
  fetchEncodedAccounts,
  isProgramError,
  isSolanaError,
  pipe,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
  TransactionSigner,
} from '@solana/kit';
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  fetchMint,
  fetchToken,
  findAssociatedTokenPda,
  getBurnInstruction,
  TOKEN_PROGRAM_ADDRESS,
} from '@solana-program/token';

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

test('can soulburn for an event', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    2,
    1n,
    endType('MaxBurns', { maxBurns: 300 })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const asset2 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 2',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [soulboundAsset2] = await findAssetPda({
    collection,
    asset: asset2,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset, asset2, soulboundAsset2],
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const assetAccounts = await fetchEncodedAccounts(client.rpc, [asset, asset2]);
  assetAccounts.forEach((acc) => t.assert(acc.exists && acc.space === 1n));

  const soulboundAssetAccounts = await fetchAllAssetV1(client.rpc, [
    soulboundAsset,
    soulboundAsset2,
  ]);

  soulboundAssetAccounts.forEach((acc) => {
    t.like(acc, <Account<AssetV1>>{
      data: {
        owner: owner.address,
        updateAuthority: {
          __kind: 'Collection',
        },
      },
    });
  });

  const token = await fetchToken(client.rpc, ata);
  t.assert(token.data.amount === 1n);

  const mintAcc = await fetchMint(client.rpc, mint);
  t.assert(mintAcc.data.supply === 1n);
});

test('cannot soulburn the same assets twice', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    2,
    1n,
    endType('MaxBurns', { maxBurns: 300 })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const asset2 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 2',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [soulboundAsset2] = await findAssetPda({
    collection,
    asset: asset2,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset, asset2, soulboundAsset2],
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  });
  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstructions([soulburnIx, soulburnIx], tx)
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__EXPECTED_MPL_CORE_ASSET
    )
  );
});

test('cannot soulburn 1 or 3 assets if expects 2', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    2,
    1n,
    endType('MaxBurns', { maxBurns: 300 })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [asset, soulboundAsset],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__INVALID_REMAINING_ACCOUNTS
    )
  );

  const asset2 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 2',
    'https://example.com'
  );

  const asset3 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 3',
    'https://example.com'
  );

  const [soulboundAsset2] = await findAssetPda({
    collection,
    asset: asset2,
  });

  const [soulboundAsset3] = await findAssetPda({
    collection,
    asset: asset3,
  });

  const transactionMessage2 = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [
            asset,
            soulboundAsset,
            asset2,
            soulboundAsset2,
            asset3,
            soulboundAsset3,
          ],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        }),
        tx
      )
  );

  const promise2 = signAndSendTransaction(client, transactionMessage2);
  const error2 = await t.throwsAsync(promise2);

  t.true(
    isSolanaError(
      error2,
      SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE
    )
  );
  t.true(
    isProgramError(
      error2.cause,
      transactionMessage,
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__INVALID_REMAINING_ACCOUNTS
    )
  );
});

test('cannot burn if max mints is reached', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    1,
    1n,
    endType('MaxBurns', { maxBurns: 1 })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const asset2 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 2',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [soulboundAsset2] = await findAssetPda({
    collection,
    asset: asset2,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset],
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [asset2, soulboundAsset2],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__BURN_EVENT_COMPLETED
    )
  );
});

test('cannot burn if max mints is reached even if they are burned', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    1,
    1n,
    endType('MaxBurns', { maxBurns: 1 })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const asset2 = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 2',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [soulboundAsset2] = await findAssetPda({
    collection,
    asset: asset2,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset],
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstructions(
        [
          soulburnIx,
          getBurnInstruction({
            mint,
            account: ata,
            amount: 1,
            authority: owner,
          }),
        ],
        tx
      ),
    (tx) => signAndSendTransaction(client, tx)
  );

  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [asset2, soulboundAsset2],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__BURN_EVENT_COMPLETED
    )
  );
});

test('cannot burn if inactive', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    1,
    1n,
    endType('MaxBurns', { maxBurns: 2 })
  );

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset],
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  });
  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx)
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__EVENT_INACTIVE
    )
  );
});

test('cannot burn if after end time', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    1,
    1n,
    endType('Timestamp', { endsAt: BigInt(Math.ceil(Date.now() / 1000) + 5) })
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 1',
    'https://example.com'
  );

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: owner.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [asset, soulboundAsset],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        }),
        tx
      ),
    (tx) => signAndSendTransaction(client, tx)
  );

  await sleep(5000);

  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getEventBurnInstruction({
          burner,
          burnEvent,
          collection,
          soulboundCollection,
          owner,
          coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
          assets: [asset, soulboundAsset],
          mint,
          ata,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__EVENT_ENDED
    )
  );
});

async function eventBurn({
  client,
  burner,
  burnEvent,
  authority,
  collection,
  owner,
  mint,
  name = 'Test asset',
  url = 'https://example.com',
}: {
  client: Client;
  burner: Address;
  burnEvent: Address;
  authority: TransactionSigner;
  collection: Address;
  owner: TransactionSigner;
  mint?: Address;
  name?: string;
  url?: string;
}) {
  const [soulboundCollection] = await findCollectionPda({
    collection,
  });
  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    name,
    url
  );

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    mint,
    ata: mint
      ? (
          await findAssociatedTokenPda({
            mint,
            owner: owner.address,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
          })
        )[0]
      : undefined,
    associatedTokenProgram: mint ? ASSOCIATED_TOKEN_PROGRAM_ADDRESS : undefined,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset],
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );
}

test('can soulburn for a noop event', async (t) => {
  t.timeout(30000);
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);
  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);
  const burnEvent = await createBurnEvent(
    client,
    authority,
    burner,
    1,
    1n,
    endType('MaxBurns', { maxBurns: 5 }),
    false
  );

  await activateBurnEvent(client, burnEvent, burner, authority);

  const owner = await generateKeyPairSignerWithSol(client);

  const assets = [
    'Test asset 1',
    'Test asset 2',
    'Test asset 3',
    'Test asset 4',
    'Test asset 5',
  ];

  await Promise.all(
    assets.map((asset) =>
      eventBurn({
        client,
        burnEvent,
        burner,
        authority,
        collection,
        owner,
        name: asset,
      })
    )
  );

  const asset = await createCoreAsset(
    client,
    authority,
    collection,
    owner.address,
    'Test asset 6',
    'http://example.com'
  );

  const [soulboundAsset] = await findAssetPda({
    collection,
    asset,
  });

  const [soulboundCollection] = await findCollectionPda({
    collection,
  });

  const soulburnIx = getEventBurnInstruction({
    burner,
    burnEvent,
    collection,
    soulboundCollection,
    owner,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    assets: [asset, soulboundAsset],
  });
  const transactionMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx)
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
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__BURN_EVENT_COMPLETED
    )
  );
});
