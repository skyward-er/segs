# https://just.systems
mod assets 'crates/segs-assets'

alias r := run
alias f := format

[private]
default:
    @just --list

run:
    cargo run --bin segs

format:
    cargo +nightly fmt
