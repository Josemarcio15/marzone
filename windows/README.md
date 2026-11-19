# Marzone (Windows 10/11)

Gerenciador de janelas **Tiling** adaptativo e editor visual de layouts para **Windows**, construído em **Rust puro** com **Slint UI** e **Win32 API**.

---

## 🚀 Recursos no Windows

1. **Daemon em Background:**
   - Roda silenciosamente na bandeja do sistema (**System Tray / Área de Notificação**).
   - Sem janelas extras de terminal ou processos pesados.

2. **Atalhos Globais Nativos (Win32 Hotkeys):**
   - **`Win + [1..9]`** (ou teclas configuradas): Encaixa a janela ativa na zona correspondente.
   - **`Win + Alt + T`**: Abre o Editor Visual de Zonas com interface moderna em Slint.
   - Suporte multi-monitor automático: a janela se encaixa no monitor em que está posicionada, respeitando a barra de tarefas (`rcWork`).

3. **Menu da Bandeja (System Tray):**
   - Clique com o botão direito no ícone do Marzone na bandeja:
     - Alternar layout ativo com 1 clique (ex: *Foco + Terminais*, *3 Colunas*, *4 Quadrantes*, *2 Metades*).
     - Abrir o Editor Visual.
     - Fechar/Sair do Marzone.
   - **Duplo clique** no ícone: Abre o Editor Visual.

4. **Mesmo Formato de Configurações:**
   - Salvo em `%APPDATA%\marzone\layouts.json` (ou `%USERPROFILE%\.config\marzone\layouts.json`).
   - Totalmente compatível com os presets e customizações do Cinnamon.

---

## 🛠️ Como Compilar e Executar

### 1. Compilar em Release
Na raiz do projeto:
```powershell
cargo build --release -p marzone-windows
```

O binário otimizado e independente é gerado em:
```text
target/release/marzone-windows.exe
```

### 2. Executar
Basta dar duplo clique em `marzone-windows.exe` ou rodar via terminal:
```powershell
.\target\release\marzone-windows.exe
```

### 3. Iniciar com o Windows (Opcional)
Para iniciar automaticamente com o Windows:
1. Pressione `Win + R`, digite `shell:startup` e pressione Enter.
2. Crie um atalho para o `marzone-windows.exe` nessa pasta.
