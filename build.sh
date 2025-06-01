#!/usr/bin/env bash
nix develop --command bash -c "
    if [ -d web/pkg ]; then rm -r web/pkg; fi &&
    wasm-pack build --target web &&
    mv pkg web/pkg"
