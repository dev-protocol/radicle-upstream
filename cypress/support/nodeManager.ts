// Copyright © 2021 The Radicle Upstream Contributors
//
// This file is part of radicle-upstream, distributed under the GPLv3
// with Radicle Linking Exception. For full terms see the included
// LICENSE file.

import qs from "qs";

import type { Config } from "ui/src/config";
import { ProxyClient } from "proxy-client";

import { createPlugin } from "cypress/support/plugin";
import {
  pluginMethods,
  NodeSession,
  NodeManagerPlugin,
} from "cypress/plugins/nodeManager/shared";

interface NodeHandle extends NodeSession {
  client: ProxyClient;
}

const nodeManagerPlugin = createPlugin<NodeManagerPlugin>(
  "nodeManager",
  pluginMethods
);

function startAndOnboardNode(
  baseDataDir: string,
  onboardedUser: OnboardedUser
): Cypress.Chainable<NodeHandle> {
  return nodeManagerPlugin
    .startNode({
      baseDataDir,
      handle: onboardedUser.handle || "secretariat",
      passphrase: onboardedUser.passphrase || "radicle-upstream",
    })
    .then(
      (nodeSession): NodeHandle => ({
        ...nodeSession,
        client: new ProxyClient(`http://localhost:${nodeSession.httpPort}`),
      })
    );
}

const withNodeManager = (callback: () => void): void => {
  nodeManagerPlugin.stopAllNodes();
  callback();
  nodeManagerPlugin.stopAllNodes();
};

interface OnboardedUser {
  handle?: string;
  passphrase?: string;
}

// Executes a shell command in the context of a node session.
//
// In particular, `.gitconfig` is properly set for the node.
export const exec = (cmd: string, session: NodeSession): void => {
  cy.exec(
    `set -euo pipefail
export PATH="$PWD/target/debug:$PATH"
${cmd}`,
    {
      env: {
        HOME: session.lnkHome,
        LNK_HOME: session.lnkHome,
        RADICLE_UNSAFE_FAST_KEYSTORE: "1",
        SSH_AUTH_SOCK: "/dev/null",
      },
    }
  );
};

export function withOneOnboardedNode(
  options: {
    dataDir: string;
    handle?: string;
  },
  callback: (node: NodeHandle) => void
): void {
  withNodeManager(() => {
    startAndOnboardNode(options.dataDir, {
      handle: options.handle,
    }).then(node => {
      callback(node);
    });
  });
}

export const asNode = (node: NodeSession): void => {
  cy.log(`switching UI to node ${node.id}`);

  const config: Partial<Config> = {
    proxyAddress: `127.0.0.1:${node.httpPort}`,
  };

  const query = qs.stringify({
    config: JSON.stringify(config),
  });
  cy.visit(`./public/index.html?${query}`);
};
