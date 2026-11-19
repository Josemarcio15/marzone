#!/usr/bin/env bash
# install_to_cinnamon.sh — Instala o Applet e o Editor Visual no Cinnamon
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

APPLET_UUID="marzone@marcio"
TARGET_DIR="$HOME/.local/share/cinnamon/applets/$APPLET_UUID"

echo "Instalando Applet e Editor Marzone em: $TARGET_DIR"
mkdir -p "$TARGET_DIR"

cp -r metadata.json applet.js tilingModel.js "$TARGET_DIR/"

# Copia o binário compilado do Slint para dentro do diretório do applet
ROOT_BIN="../target/release/marzone-editor"
LOCAL_BIN="editor/target/release/marzone-editor"

if [ -f "$ROOT_BIN" ]; then
    echo "Copiando binário do editor Slint de $ROOT_BIN..."
    cp -f "$ROOT_BIN" "$TARGET_DIR/marzone-editor.tmp"
    mv -f "$TARGET_DIR/marzone-editor.tmp" "$TARGET_DIR/marzone-editor"
    chmod +x "$TARGET_DIR/marzone-editor"
elif [ -f "$LOCAL_BIN" ]; then
    echo "Copiando binário do editor Slint de $LOCAL_BIN..."
    cp -f "$LOCAL_BIN" "$TARGET_DIR/marzone-editor.tmp"
    mv -f "$TARGET_DIR/marzone-editor.tmp" "$TARGET_DIR/marzone-editor"
    chmod +x "$TARGET_DIR/marzone-editor"
fi

echo "Instalação concluída com sucesso!"
echo ""
echo "Atalhos configurados:"
echo " • Super + Alt + T: Abre o Editor Visual de Zonas"
echo " • Super + [1..9]: Encaixa a janela focada na zona correspondente"
