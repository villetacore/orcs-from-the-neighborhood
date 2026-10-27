#!/bin/sh
cd -- "$(dirname -- "$0")" || exit 1
if command -v cargo >/dev/null 2>&1; then
    exec cargo run --release
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    exec "$HOME/.cargo/bin/cargo" run --release
else
    echo 'Установите Rust (https://rustup.rs) и запустите снова.'
    exit 1
fi
