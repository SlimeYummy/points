SET RUST_BACKTRACE=1
@REM SET RUST_LOG="debug,wasmtime=info,cranelift_codegen=info,wasmtime_cranelift=info"
@REM SET WASMTIME_LOG=wasmtime=info
cargo run -- --template \project\points\test-tmp\demo-template --asset \project\points\test-tmp\test-asset --save \project\points\test-tmp\demo-save
