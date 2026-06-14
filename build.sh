#!/bin/sh
TARGET=$(rustc -Vv | grep host | cut -d ' ' -f 2)

cargo +nightly build --release \
  --target "$TARGET" \
  -Zbuild-std=std,panic_abort \
  --config "target.$TARGET.rustflags='-Zunstable-options -Cpanic=immediate-abort -Zfmt-debug=none'"

mkdir -p ./target/release
cp "./target/$TARGET/release/athan" ./target/release/athan
