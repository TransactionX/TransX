#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ] ; then
   rustup install nightly-2020-03-09
   rustup default nightly-2020-03-09-x86_64-unknown-linux-gnu
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2020-03-09-x86_64-unknown-linux-gnu
