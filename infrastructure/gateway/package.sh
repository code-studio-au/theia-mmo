#!/usr/bin/env bash
set -euo pipefail

cargo build --release --locked --manifest-path infrastructure/gateway/Cargo.toml \
  --target x86_64-unknown-linux-musl -j 2
target=infrastructure/gateway/target/x86_64-unknown-linux-musl/release
output=build/realm-gateway
mkdir -p "$output"
rm -f "$output/bootstrap.zip"
zip -j "$output/bootstrap.zip" "$target/bootstrap"
install -m 0755 "$target/theia-realm-host" "$output/theia-realm-host"
(
  cd "$output"
  sha256sum bootstrap.zip theia-realm-host > SHA256SUMS
)
