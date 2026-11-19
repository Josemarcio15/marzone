#!/usr/bin/env bash
set -euo pipefail

echo "Compilando o Editor Visual em Slint + Rust..."
cd "$(dirname "$0")"

cargo build --release

echo ""
echo "Compilação concluída com sucesso!"
echo "Executável gerado em: target/release/marzone-editor"
