import test from 'ava';
import {
  findMintPda,
  getAdminMintInstruction,
  SOULBURN_ERROR__MAX_TOKENS_MINTED,
  SOULBURN_PROGRAM_ADDRESS,
} from '../src';
import {
  createBurner,
  createBurnEvent,
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreCollection } from './_mpl-core';
import {
  appendTransactionMessageInstruction,
  isProgramError,
  isSolanaError,
  pipe,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
} from '@solana/kit';
import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  fetchMint,
  fetchToken,
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
} from '@solana-program/token';

test('can mint tokens for an event', async (t) => {
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
    300n
  );

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: authority.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getAdminMintInstruction({
    burner,
    burnEvent,
    owner: authority.address,
    authority,
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    amount: 300n,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const token = await fetchToken(client.rpc, ata);
  t.assert(token.data.amount === 300n);

  const mintAcc = await fetchMint(client.rpc, mint);
  t.assert(mintAcc.data.supply === 300n);
});

test('cannot mint more tokens than the event config', async (t) => {
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
    300n
  );

  const [mint] = await findMintPda({
    burnEvent,
  });

  const [ata] = await findAssociatedTokenPda({
    mint,
    owner: authority.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  // When we create a new burner account.
  const soulburnIx = getAdminMintInstruction({
    burner,
    burnEvent,
    owner: authority.address,
    authority,
    mint,
    ata,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
    amount: 301n,
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
      SOULBURN_ERROR__MAX_TOKENS_MINTED
    )
  );
});
