cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release
echo To compress with upx, uncomment the line below.
rem "./target/upx.exe" --all-methods --all-filters ./"target/i586-pc-windows-msvc/release/hello-rust.exe"