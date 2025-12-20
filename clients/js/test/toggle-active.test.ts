import test from 'ava';
import {
  BurnEvent,
  endType,
  fetchBurnEvent,
  getToggleEventActiveInstruction,
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
  Account,
  appendTransactionMessageInstruction,
  pipe,
} from '@solana/kit';

test('it creates a new burn event account', async (t) => {
  t.timeout(30000);
  // Given an authority key pair with some SOL.
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

  // When we create a new burner account.
  const soulburnIx = getToggleEventActiveInstruction({
    burner,
    burnEvent,
    authority,
    active: true,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(soulburnIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const burnEventAccount = await fetchBurnEvent(client.rpc, burnEvent);

  t.like(burnEventAccount, <Account<BurnEvent>>{
    data: {
      active: true,
    },
  });
});
