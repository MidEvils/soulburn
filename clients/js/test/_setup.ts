import {
  Address,
  Commitment,
  TransactionSigner,
  TransactionMessageWithBlockhashLifetime,
  Rpc,
  RpcSubscriptions,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
  airdropFactory,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getSignatureFromTransaction,
  lamports,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  TransactionMessage,
  TransactionMessageWithFeePayer,
  SendableTransaction,
  Transaction,
  TransactionBlockhashLifetime,
  generateKeyPair,
  createSignerFromKeyPair,
  appendTransactionMessageInstruction,
  KeyPairSigner,
  generateKeyPairSigner,
} from '@solana/kit';
import { MPL_CORE_PROGRAM_PROGRAM_ADDRESS } from '../sdks/mpl-core/generated';
import {
  findBurnerPda,
  findCollectionPda,
  findMintPda,
  getCreateBurnEventInstruction,
  getCreateInstruction,
  getToggleEventActiveInstruction,
} from '../src';

export type Client = {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
};

export const createDefaultSolanaClient = (): Client => {
  const rpc = createSolanaRpc('http://127.0.0.1:8899');
  const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
  return { rpc, rpcSubscriptions };
};

export const generateKeyPairSignerWithSol = async (
  client: Client,
  putativeLamports: bigint = 1_000_000_000n
) => {
  const keypair = await generateKeyPair();
  const signer = await createSignerFromKeyPair(keypair);
  await airdropFactory(client)({
    recipientAddress: signer.address,
    lamports: lamports(putativeLamports),
    commitment: 'confirmed',
  });
  return signer;
};

export const createDefaultTransaction = async (
  client: Client,
  feePayer: TransactionSigner
) => {
  const { value: latestBlockhash } = await client.rpc
    .getLatestBlockhash()
    .send();
  return pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(feePayer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx)
  );
};

export const signAndSendTransaction = async (
  client: Client,
  transactionMessage: TransactionMessage &
    TransactionMessageWithFeePayer &
    TransactionMessageWithBlockhashLifetime,
  commitment: Commitment = 'confirmed'
) => {
  const signedTransaction = (await signTransactionMessageWithSigners(
    transactionMessage
  )) as SendableTransaction &
    Transaction & {
      readonly lifetimeConstraint: TransactionBlockhashLifetime;
    };
  const signature = getSignatureFromTransaction(signedTransaction);
  await sendAndConfirmTransactionFactory(client)(signedTransaction, {
    commitment,
  });
  return signature;
};

export const getBalance = async (client: Client, address: Address) =>
  (await client.rpc.getBalance(address, { commitment: 'confirmed' }).send())
    .value;

export const createBurner = async (
  client: Client,
  authority: KeyPairSigner,
  collection: Address
) => {
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

  return burner;
};

export const createBurnEvent = async (
  client: Client,
  authority: KeyPairSigner,
  burner: Address,
  burnsRequired: number,
  tokensPerEventBurn: bigint,
  maxTokens?: bigint,
  endsAt?: bigint
): Promise<Address> => {
  const burnEvent = await generateKeyPairSigner();

  const [mint] = await findMintPda({ burnEvent: burnEvent.address });

  // When we create a new burn-event account.
  const createIx = getCreateBurnEventInstruction({
    burnEvent,
    authority,
    burner,
    burnsRequired,
    endsAt: endsAt || null,
    maxTokens: maxTokens || null,
    mint,
    tokensPerEventBurn,
  });
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) => appendTransactionMessageInstruction(createIx, tx),
    (tx) => signAndSendTransaction(client, tx)
  );

  return burnEvent.address;
};

export const activateBurnEvent = async (
  client: Client,
  burnEvent: Address,
  burner: Address,
  authority: KeyPairSigner
) => {
  await pipe(
    await createDefaultTransaction(client, authority),
    (tx) =>
      appendTransactionMessageInstruction(
        getToggleEventActiveInstruction({
          burner,
          burnEvent,
          authority,
          active: true,
        }),
        tx
      ),
    (tx) => signAndSendTransaction(client, tx)
  );
};
