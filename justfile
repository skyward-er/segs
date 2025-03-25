alias r := run
alias t := test
alias d := doc

default:
    just run

test *ARGS:
    cargo nextest run {{ARGS}}

run LEVEL="debug":
    RUST_BACKTRACE=full RUST_LOG=segs={{LEVEL}} cargo r

package:
    cargo packager --release

doc:
    cargo doc --no-deps --open
