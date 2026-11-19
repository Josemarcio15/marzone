// marzone-windows: Daemon de tiling para Windows 10 em Rust puro com Slint
slint::include_modules!();

fn main() {
    println!("Marzone Windows daemon iniciado.");
    // No Windows: registra hooks de atalhos globais (SetWindowsHookEx) e System Tray
    // Quando solicitado (ex: Tray -> 'Editar Zonas'), exibe MainWindow::new().unwrap().run();
}
