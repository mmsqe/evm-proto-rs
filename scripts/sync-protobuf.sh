#!/usr/bin/env bash

set -eou pipefail

CACHE_PATH="${XDG_CACHE_HOME:-$HOME/.cache}"/evm-proto-rs-build
EVM_GIT="${EVM_GIT:-$CACHE_PATH/cosmos/evm.git}"
COSMOS_SDK_GIT="${COSMOS_SDK_GIT:-$CACHE_PATH/cosmos/cosmos-sdk.git}"

EVM_COMMIT="$(cat src/EVM_COMMIT)"
COSMOS_SDK_COMMIT="$(cat src/COSMOS_SDK_COMMIT 2>/dev/null || echo "v0.53.3")"

echo "EVM_COMMIT: $EVM_COMMIT"
echo "COSMOS_SDK_COMMIT: $COSMOS_SDK_COMMIT"


if [[ ! -e "$EVM_GIT" ]]
then
    echo "Cloning evm source code to as bare git repository to $EVM_GIT"
    git clone --mirror https://github.com/cosmos/evm "$EVM_GIT"
else
    echo "Using existing evm bare git repository at $EVM_GIT"
fi

if [[ ! -e "$COSMOS_SDK_GIT" ]]
then
    echo "Cloning cosmos-sdk source code to as bare git repository to $COSMOS_SDK_GIT"
    git clone --mirror https://github.com/cosmos/cosmos-sdk "$COSMOS_SDK_GIT"
else
    echo "Using existing cosmos-sdk bare git repository at $COSMOS_SDK_GIT"
fi

# Update the repositories using git fetch. This is so that
# we keep local copies of the repositories up to sync first.

pushd "$EVM_GIT"
git fetch
popd

pushd "$COSMOS_SDK_GIT"
git fetch
popd

# Create a new temporary directory to check out the
# actual source files from the bare git repositories.
# This is so that we do not accidentally use an unclean
# local copy of the source files to generate the protobuf.

EVM_DIR=$(mktemp -d /tmp/evm-XXXXXXXX)
COSMOS_SDK_DIR=$(mktemp -d /tmp/cosmos-sdk-XXXXXXXX)

pushd "$EVM_DIR"
git clone "$EVM_GIT" .
git switch -c "$EVM_COMMIT"

cd proto
buf export -v -o ../proto-include
popd

pushd "$COSMOS_SDK_DIR"
git clone "$COSMOS_SDK_GIT" .
git checkout "$COSMOS_SDK_COMMIT"

cd proto
buf export -v -o ../proto-include
popd

# Remove the existing generated protobuf files
# so that the newly generated code does not
# contain removed files.

PROST_DIR="prost"

rm -rf "src/$PROST_DIR"
mkdir -p "src/$PROST_DIR"

# Build the proto compiler
cargo build --release

# Run the proto-compiler with transport support enabled

cargo run --release -- compile \
  --transport \
  --evm "$EVM_DIR/proto-include" \
  --cosmos-sdk "$COSMOS_SDK_DIR/proto-include" \
  --out "src/$PROST_DIR"

# Remove leftover Cosmos SDK modules.
rm -f "src/$PROST_DIR/cosmos.base.store.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos.auth.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos.base.query.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos.base.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos.staking.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos.upgrade.v1beta1.rs"
rm -f "src/$PROST_DIR/cosmos_proto.rs"

# The Tendermint ABCI protos are unused from within evm-proto
rm -f "src/$PROST_DIR/tendermint.abci.rs"

# Remove leftover Google HTTP configuration protos.
rm -f "src/$PROST_DIR/google.api.rs"

# Remove the temporary checkouts of the repositories
rm -rf "$EVM_DIR"
rm -rf "$COSMOS_SDK_DIR"
