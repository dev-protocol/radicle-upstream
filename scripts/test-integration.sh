#!/bin/bash

# Copyright © 2022 The Radicle Upstream Contributors
#
# This file is part of radicle-upstream, distributed under the GPLv3
# with Radicle Linking Exception. For full terms see the included
# LICENSE file.

set -euo pipefail

export TZ=UTC

function kill_jobs() {
  for pid in $(jobs -rp); do
    kill -s SIGTERM "$pid" || true
  done
}

trap kill_jobs ERR EXIT

declare -ar proxy_args=(
  --test
  --unsafe-fast-keystore
  --http-listen "127.0.0.1:30000"
)

function run () {
  yarn run webpack build --config-name ui

  cargo build --all-features --bins
  cargo run --all-features --bin upstream-proxy \
    -- "${proxy_args[@]}" &

  yarn run wait-on tcp:127.0.0.1:30000
  yarn run cypress run

  kill %1
}

function run_debug () {
  if [[ -f "./public/bundle.js" ]]; then
    rm ./public/bundle.js
  fi

  yarn run webpack watch --config-name ui &
  cargo build --all-features --bins
  cargo watch -x "run --all-features -- ${proxy_args[*]}" &
  yarn run wait-on tcp:127.0.0.1:30000 ./public/bundle.js
  yarn run cypress open &

  wait -n
  kill_jobs
  wait
}

if [[ "${1:-}" == "--debug" ]]; then
  run_debug
elif [[ -z "${1:-}" ]]; then
  run
else
  echo "invalid argument"
  exit 1
fi
