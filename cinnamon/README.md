# Marzone (Cinnamon Applet)

Gerenciador de janelas **Tiling** adaptativo para o ambiente desktop **Cinnamon** (Linux Mint / LMDE).

## Recursos

1. **Cálculo de Borda Compartilhada (`computeZoneRect`):**
   * Matemática normalizada (`0.0` a `1.0`) para evitar sobreposição ou falhas de 1px entre janelas adjacentes.
2. **Layout Presets Embutidos:**
   * **Foco + Terminais (`focus-code`):** Janela principal a 65% na esquerda e 2 slots verticais a 35% na direita.
   * **3 Colunas (`3-columns`):** Esquerda (1/3), Centro (1/3), Direita (1/3).
   * **4 Quadrantes (`quarters`):** Telas divididas em 4 cantos (`1`, `3`, `7`, `9`).
   * **2 Metades (`halves`):** Split clássico 50/50.
3. **Atalhos Dinâmicos:**
   * `Super + 1..9`: Encaixa a janela ativa na zona correspondente do layout selecionado.
   * `Super + Alt + T`: Abre o Editor Visual de Zonas em Rust + Slint.
4. **Applet no Painel:**
   * Ícone no painel do Cinnamon com menu drop-down para alternar o layout ativo com 1 clique e acionar zonas manualmente.

---

## Como Compilar e Usar no Cinnamon

### 1. Compilar o Editor Visual (Rust + Slint)
Na raiz do repositório:

```bash
cargo build --release -p marzone-editor
```
*(O binário otimizado será gerado em `cinnamon/editor/target/release/marzone-editor`)*

---

### 2. Instalar o Applet e o Editor
Basta rodar o instalador:

```bash
cd ~/git/marzone/cinnamon
./install.sh
```

---

### 3. Ativar e Usar

1. Pressione **`Alt + F2`**, digite **`r`** e aperte **Enter** (para recarregar o Cinnamon).
2. Clique com o botão direito no painel do Cinnamon -> **Applets**.
3. Procure por **Marzone** e adicione-o ao painel.

---

### 4. Atalhos no Cinnamon

- **`Super + [1..9]`**: Encaixa a janela ativa na zona correspondente do layout selecionado.
- **`Super + Alt + T`**: Abre instantaneamente o Editor Visual de Zonas em Rust/Slint.
- **Clique no ícone do painel**: Abre o menu de seleção rápida de layouts e zonas.
