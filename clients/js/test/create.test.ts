import test from 'ava';
import {
  Burner,
  fetchBurnerFromSeeds,
  findBurnerPda,
  findCollectionPda,
  getCreateInstruction,
} from '../src';
import {
  createDefaultSolanaClient,
  createDefaultTransaction,
  generateKeyPairSignerWithSol,
  signAndSendTransaction,
} from './_setup';
import { createCoreCollection } from './_mpl-core';
import {
  CollectionV1,
  MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
  fetchCollectionV1,
} from '../sdks/mpl-core/generated';
import {
  Account,
  appendTransactionMessageInstruction,
  pipe,
} from '@solana/kit';

test('it creates a new burner account', async (t) => {
  t.timeout(30000);
  // Given an authority key pair with some SOL.
  const client = createDefaultSolanaClient();
  const authority = await generateKeyPairSignerWithSol(client);

  const collection = await createCoreCollection(client, authority);

  const [burner] = await findBurnerPda({ collection });
  const [soulboundCollection] = await findCollectionPda({ collection });

  const name = 'Soulbound MidEvils';
  const uri = 'https://example.com';

  // When we create a new burner account.
  const createIx = getCreateInstruction({
    authority,
    burner,
    collection,
    soulboundCollection,
    coreProgram: MPL_CORE_PROGRAM_PROGRAM_ADDRESS,
    name,
    uri,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(createIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  // Then we expect the burner account to exist .
  const burnerAccount = await fetchBurnerFromSeeds(client.rpc, {
    collection,
  });
  t.like(burnerAccount, <Account<Burner>>{
    data: {
      authority: authority.address,
      collection,
    },
  });

  // and we expect the new collection to be created

  const soulboundCollectionAccount = await fetchCollectionV1(
    client.rpc,
    soulboundCollection
  );

  t.like(soulboundCollectionAccount, <Account<CollectionV1>>{
    data: {
      numMinted: 0,
      currentSize: 0,
      uri,
      name,
      updateAuthority: burner,
    },
  });
});
