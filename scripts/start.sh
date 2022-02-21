#!/bin/bash

# Copyright © 2022 The Radicle Upstream Contributors
#
# This file is part of radicle-upstream, distributed under the GPLv3
# with Radicle Linking Exception. For full terms see the included
# LICENSE file.

set -euo pipefail

function kill_jobs() {
  for pid in $(jobs -rp); do
    kill -s SIGTERM "$pid" || true
  done
}

trap kill_jobs ERR EXIT

cargo build --all-features --bins

if [[ -f "./public/bundle.js" ]]; then
  rm ./public/bundle.js
fi

yarn run webpack watch --config-name ui &
yarn run wait-on ./public/bundle.js
yarn run electron native/index.js &

wait -n
kill_jobs
wait
