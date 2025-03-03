alias r := run
alias t := test
alias d := doc

default:
    just run

test *ARGS:
    cargo nextest run {{ARGS}}

run LEVEL="debug":
    RUST_LOG=segs={{LEVEL}} cargo r

doc:
    cargo doc --no-deps --open
