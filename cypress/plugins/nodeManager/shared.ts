// Copyright © 2021 The Radicle Upstream Contributors
//
// This file is part of radicle-upstream, distributed under the GPLv3
// with Radicle Linking Exception. For full terms see the included
// LICENSE file.

// Types and constants shared between `plugin.ts` and `commands.ts`. These
// types can't be in `plugin.ts`, because Cypress plugins run in a separate
// Nodejs environment and can directly use Nodejs libraries, the Cypress tests
// don't have access to those, so indlucing `plugin.ts` inside `commands.ts`
// leads to errors.

export type PeerId = string;

export interface NodeSession {
  id: number;
  peerId: PeerId;
  authToken: string;
  httpPort: number;
  radHome: string;
}

export type NodeId = number;

export interface OnboardNodeOptions {
  id: NodeId;
  handle: string;
  passphrase: string;
}

// We us `Promise<null>` because Cypress complains if we use
// `Promise<void>` or `Promise<undefined>`.
//
// See https://docs.cypress.io/api/commands/task.html#Usage
export interface NodeManagerPlugin {
  // Start a node and return the node’s ID which is also the port it
  // the API is listening on.
  //
  // The directory `${dataDir}/node-${id}` will be used to store node
  // related data.
  startNode: (dataDir: string) => Promise<number>;
  onboardNode: (options: OnboardNodeOptions) => Promise<NodeSession>;
  stopAllNodes: () => Promise<null>;
}

export const pluginMethods: Array<keyof NodeManagerPlugin> = [
  "startNode",
  "onboardNode",
  "stopAllNodes",
];
