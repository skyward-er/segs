# https://just.systems
mod assets 'crates/segs-assets'

alias r := run
alias f := format
alias c := clear-metadata

[private]
default:
    @just --list

run:
    cargo run --bin segs

format:
    cargo +nightly fmt

[confirm]
clear-metadata:
    rm -r ~/Library/Application\ Support/eu.skyward.segs/metadata_dev
