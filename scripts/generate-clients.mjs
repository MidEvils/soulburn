#!/usr/bin/env zx
import 'zx/globals';
import * as c from 'codama';
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { getAllProgramIdls } from './utils.mjs';

// Instanciate Codama.
const [idl, ...additionalIdls] = getAllProgramIdls().map((idl) =>
  rootNodeFromAnchor(require(idl))
);
const codama = c.createFromRoot(idl, additionalIdls);

// Update programs.
codama.update(
  c.updateProgramsVisitor({
    midevilsSoulburn: { name: 'soulburn' },
  })
);

// Update accounts.
codama.update(
  c.updateAccountsVisitor({
    burner: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'soulburn'),
        c.constantPdaSeedNodeFromString('utf8', 'burner'),
        c.variablePdaSeedNode(
          'collection',
          c.publicKeyTypeNode(),
          'The collection of the burner account'
        ),
      ],
    },
    collection: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'soulburn'),
        c.constantPdaSeedNodeFromString('utf8', 'collection'),
        c.variablePdaSeedNode(
          'collection',
          c.publicKeyTypeNode(),
          'The collection of the burner account'
        ),
      ],
    },
    asset: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'soulburn'),
        c.constantPdaSeedNodeFromString('utf8', 'asset'),
        c.variablePdaSeedNode(
          'collection',
          c.publicKeyTypeNode(),
          'The collection of the burner account'
        ),
        c.variablePdaSeedNode(
          'asset',
          c.publicKeyTypeNode(),
          'The soulburned asset'
        ),
      ],
    },
    mint: {
      seeds: [
        c.constantPdaSeedNodeFromString('utf8', 'soulburn'),
        c.constantPdaSeedNodeFromString('utf8', 'mint'),
        c.variablePdaSeedNode(
          'burn_event',
          c.publicKeyTypeNode(),
          'The burn_event account'
        ),
      ],
    },
  })
);

// Update instructions.
codama.update(
  c.updateInstructionsVisitor({
    create: {
      byteDeltas: [c.instructionByteDeltaNode(c.accountLinkNode('burner'))],
      accounts: {
        burner: { defaultValue: c.pdaValueNode('burner') },
        payer: { defaultValue: c.accountValueNode('authority') },
      },
    },
    eventBurn: {
      remainingAccounts: [
        c.instructionRemainingAccountsNode(c.argumentValueNode('assets'), {
          isWritable: true,
        }),
      ],
    },
    soulburnAsset: {
      accounts: {
        burner: { defaultValue: c.pdaValueNode('burner') },
      },
    },
  })
);

// Set account discriminators.
const key = (name) => ({ field: 'key', value: c.enumValueNode('Key', name) });
codama.update(
  c.setAccountDiscriminatorFromFieldVisitor({
    burner: key('burner'),
    burnEvent: key('burnEvent'),
  })
);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettierOptions: require(path.join(jsClient, '.prettierrc.json')),
  })
);
