import test from 'ava';
import {
  BurnEvent,
  fetchBurnEvent,
  findMintPda,
  getCreateBurnEventInstruction,
  Key,
} from '../src';
import {
  createBurner,
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreCollection } from './_mpl-core';

import {
  Account,
  appendTransactionMessageInstruction,
  generateKeyPairSigner,
  none,
  pipe,
} from '@solana/kit';

test('it creates a new burn event account', async (t) => {
  t.timeout(30000);
  // Given an authority key pair with some SOL.
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);

  const collection = await createCoreCollection(client, authority);

  const burner = await createBurner(client, authority, collection);

  const burnEvent = await generateKeyPairSigner();

  const [mint] = await findMintPda({
    burnEvent: burnEvent.address,
  });

  // When we create a new burn-event account.
  const createIx = getCreateBurnEventInstruction({
    burnEvent,
    authority,
    burner,
    burnsRequired: 2,
    endsAt: null,
    maxTokens: 300,
    mint,
    tokensPerEventBurn: 1n,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(createIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  const burnEventAccount = await fetchBurnEvent(client.rpc, burnEvent.address);

  t.like(burnEventAccount, <Account<BurnEvent>>{
    data: {
      key: Key.BurnEvent,
      burner,
      active: false,
      endsAt: none(),
      burnsRequired: 2,
    },
  });
});
