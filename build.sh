#!/usr/bin/env bash
nix develop --command bash -c "
    rm -r web/pkg &&
    wasm-pack build --target web &&
    mv pkg web/pkg"
