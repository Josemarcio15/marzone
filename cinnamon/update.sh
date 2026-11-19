#!/usr/bin/env bash
# update.sh — Compila e instala tudo em 1 único comando
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== 1. Compilando Editor Slint + Rust (Release) ==="
cd editor-slint
cargo build --release
cd ..

echo "=== 2. Instalando Applet no Cinnamon ==="
./install.sh

echo ""
echo "=== Tudo pronto! ==="
echo "Pressione Alt + F2, digite 'r' e tecle Enter se alterou o applet.js."
