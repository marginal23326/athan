$Target = (rustc -Vv | Select-String "host:").Line.Split(" ")[1]

cargo +nightly build --release `
  --target $Target `
  "-Zbuild-std=std,panic_abort" `
  --config "target.$Target.rustflags='-Zunstable-options -Cpanic=immediate-abort'"

New-Item -ItemType Directory -Force -Path "./target/release" | Out-Null
Copy-Item "./target/$Target/release/athan.exe" "./target/release/athan.exe" -Force
