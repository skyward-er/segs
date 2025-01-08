alias r := run
alias d := doc

default:
    just run

run LEVEL="debug":
    RUST_LOG=segs={{LEVEL}} cargo r

doc:
    cargo doc --no-deps --open
