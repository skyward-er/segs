# https://just.systems
mod assets 'crates/segs-assets'

alias r := run

[private]
default:
    @just --list

run:
    cargo run --bin segs
