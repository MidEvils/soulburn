import test from 'ava';
import {
  activateBurnEvent,
  createBurner,
  createBurnEvent,
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreCollection } from './_mpl-core';
import {
  endType,
  getDeleteBurnEventInstruction,
  SOULBURN_ERROR__ACCOUNT_MISMATCH,
  SOULBURN_PROGRAM_ADDRESS,
} from '../src';
import {
  appendTransactionMessageInstruction,
  isProgramError,
  isSolanaError,
  pipe,
  SOLANA_ERROR__JSON_RPC__SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE,
} from '@solana/kit';

test('can delete a burn event', async (t) => {
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

  const ix = getDeleteBurnEventInstruction({
    authority,
    burner,
    burnEvent,
  });

  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(ix, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const { value } = await client.rpc.getAccountInfo(burnEvent).send();

  t.deepEqual(value, null);
});

test('cannot delete a burn event if not admin', async (t) => {
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

  const nonAuth = await generateKeyPairSignerWithSol(client);

  const ix = getDeleteBurnEventInstruction({
    authority: nonAuth,
    burner,
    burnEvent,
  });

  const txMessage = pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(ix, tx)
  );

  const promise = signAndSendTransaction(client, txMessage);

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
      txMessage,
      SOULBURN_PROGRAM_ADDRESS,
      SOULBURN_ERROR__ACCOUNT_MISMATCH
    )
  );
});
