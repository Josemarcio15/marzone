// marzone-windows: Daemon de tiling para Windows em Rust puro com Slint
#![windows_subsystem = "windows"]

use serde::{Deserialize, Serialize};
use slint::{Color, ComponentHandle, Model, ModelRc, RgbaColor, VecModel};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateBitmap, DeleteObject, GetDC, GetMonitorInfoW, MonitorFromWindow, ReleaseDC,
    HBITMAP, HDC, HMONITOR, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState,
    VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9,
    VK_A, VK_B, VK_C, VK_D, VK_E, VK_F, VK_G, VK_H, VK_I, VK_J,
    VK_K, VK_L, VK_M, VK_N, VK_O, VK_P, VK_Q, VK_R, VK_S, VK_T,
    VK_U, VK_V, VK_W, VK_X, VK_Y, VK_Z,
    VK_LWIN, VK_RWIN, VK_LMENU, VK_RMENU, VK_LCONTROL, VK_RCONTROL,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIM_ADD, NIM_DELETE, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos,
    GetForegroundWindow, GetMessageW, InsertMenuW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SetForegroundWindow, SetWindowPos,
    SetWindowsHookExW, ShowWindow, TrackPopupMenu, UnhookWindowsHookEx,
    CS_HREDRAW, CS_VREDRAW, HHOOK, HICON, HMENU, ICONINFO, KBDLLHOOKSTRUCT,
    MF_BYPOSITION, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG,
    SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON,
    WH_KEYBOARD_LL, WM_COMMAND, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_USER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

slint::include_modules!();

const WM_TRAYICON: u32 = WM_USER + 100;
const WM_APP_OPEN_EDITOR: u32 = WM_USER + 101;
const WM_APP_TILE_ZONE: u32 = WM_USER + 102;

const IDM_OPEN_EDITOR: usize = 2001;
const IDM_AUTOSTART: usize = 2002;
const IDM_EXIT: usize = 2003;
const IDM_LAYOUT_BASE: usize = 3000;

const VIBRANT_PALETTE: &[(u8, u8, u8)] = &[
    (79, 140, 255),
    (46, 204, 113),
    (241, 196, 15),
    (230, 126, 34),
    (231, 76, 60),
    (155, 89, 182),
    (26, 188, 156),
    (255, 107, 129),
    (52, 152, 219),
    (0, 210, 211),
];

fn get_color_for_id(id: i32) -> Color {
    let idx = (id.abs() as usize) % VIBRANT_PALETTE.len();
    let (r, g, b) = VIBRANT_PALETTE[idx];
    Color::from(RgbaColor {
        red: r,
        green: g,
        blue: b,
        alpha: 255,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZoneConfig {
    pub id: i32,
    pub key: String,
    pub title: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayoutConfig {
    pub id: String,
    pub name: String,
    pub zones: Vec<ZoneConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    #[serde(rename = "activeLayoutId")]
    pub active_layout_id: String,
    #[serde(default = "default_gap")]
    pub gap: i32,
    pub layouts: HashMap<String, LayoutConfig>,
}

fn default_gap() -> i32 {
    6
}

fn snap(val: f32) -> f32 {
    (val * 20.0).round() / 20.0
}

fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("marzone").join("layouts.json")
}

fn default_layouts() -> HashMap<String, LayoutConfig> {
    let mut layouts = HashMap::new();
    layouts.insert(
        "focus-code".to_string(),
        LayoutConfig {
            id: "focus-code".to_string(),
            name: "Foco + Terminais".to_string(),
            zones: vec![
                ZoneConfig {
                    id: 1,
                    key: "1".into(),
                    title: "Editor Principal (65%)".into(),
                    x: 0.0,
                    y: 0.0,
                    w: 0.65,
                    h: 1.0,
                    color: None,
                },
                ZoneConfig {
                    id: 2,
                    key: "2".into(),
                    title: "Terminal Superior (35%)".into(),
                    x: 0.65,
                    y: 0.0,
                    w: 0.35,
                    h: 0.5,
                    color: None,
                },
                ZoneConfig {
                    id: 3,
                    key: "3".into(),
                    title: "Terminal Inferior (35%)".into(),
                    x: 0.65,
                    y: 0.5,
                    w: 0.35,
                    h: 0.5,
                    color: None,
                },
            ],
        },
    );
    layouts.insert(
        "3-columns".to_string(),
        LayoutConfig {
            id: "3-columns".to_string(),
            name: "3 Colunas".to_string(),
            zones: vec![
                ZoneConfig {
                    id: 1,
                    key: "1".into(),
                    title: "Esquerda (1/3)".into(),
                    x: 0.0,
                    y: 0.0,
                    w: 0.3333,
                    h: 1.0,
                    color: None,
                },
                ZoneConfig {
                    id: 2,
                    key: "2".into(),
                    title: "Centro (1/3)".into(),
                    x: 0.3333,
                    y: 0.0,
                    w: 0.3333,
                    h: 1.0,
                    color: None,
                },
                ZoneConfig {
                    id: 3,
                    key: "3".into(),
                    title: "Direita (1/3)".into(),
                    x: 0.6666,
                    y: 0.0,
                    w: 0.3334,
                    h: 1.0,
                    color: None,
                },
            ],
        },
    );
    layouts.insert(
        "quarters".to_string(),
        LayoutConfig {
            id: "quarters".to_string(),
            name: "4 Quadrantes".to_string(),
            zones: vec![
                ZoneConfig {
                    id: 7,
                    key: "7".into(),
                    title: "Sup. Esquerdo".into(),
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5,
                    color: None,
                },
                ZoneConfig {
                    id: 9,
                    key: "9".into(),
                    title: "Sup. Direito".into(),
                    x: 0.5,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5,
                    color: None,
                },
                ZoneConfig {
                    id: 1,
                    key: "1".into(),
                    title: "Inf. Esquerdo".into(),
                    x: 0.0,
                    y: 0.5,
                    w: 0.5,
                    h: 0.5,
                    color: None,
                },
                ZoneConfig {
                    id: 3,
                    key: "3".into(),
                    title: "Inf. Direito".into(),
                    x: 0.5,
                    y: 0.5,
                    w: 0.5,
                    h: 0.5,
                    color: None,
                },
            ],
        },
    );
    layouts.insert(
        "halves".to_string(),
        LayoutConfig {
            id: "halves".to_string(),
            name: "2 Metades (Split)".to_string(),
            zones: vec![
                ZoneConfig {
                    id: 1,
                    key: "1".into(),
                    title: "Esquerda".into(),
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 1.0,
                    color: None,
                },
                ZoneConfig {
                    id: 2,
                    key: "2".into(),
                    title: "Direita".into(),
                    x: 0.5,
                    y: 0.0,
                    w: 0.5,
                    h: 1.0,
                    color: None,
                },
            ],
        },
    );
    layouts
}

fn load_config() -> ConfigFile {
    let path = config_path();
    let defaults = default_layouts();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(mut parsed) = serde_json::from_str::<ConfigFile>(&content) {
            for (k, v) in defaults {
                parsed.layouts.entry(k).or_insert(v);
            }
            if !parsed.layouts.contains_key(&parsed.active_layout_id) {
                parsed.active_layout_id = parsed
                    .layouts
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "focus-code".to_string());
            }
            return parsed;
        }
    }

    ConfigFile {
        active_layout_id: "focus-code".to_string(),
        gap: 6,
        layouts: defaults,
    }
}

fn save_config(cfg: &ConfigFile) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, json);
    }
}

// Helpers de Inicialização com o Windows (HKCU Run Registry)
const RUN_KEY_PATH: windows::core::PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const APP_REG_NAME: windows::core::PCWSTR = w!("Marzone");

fn is_autostart_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, 0, KEY_QUERY_VALUE, &mut hkey).is_ok() {
            let mut buf = [0u16; 512];
            let mut len = (buf.len() * 2) as u32;
            let res = RegQueryValueExW(
                hkey,
                APP_REG_NAME,
                None,
                None,
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut len),
            );
            let _ = RegCloseKey(hkey);
            res.is_ok()
        } else {
            false
        }
    }
}

fn set_autostart(enable: bool) {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY_PATH, 0, KEY_SET_VALUE, &mut hkey).is_ok() {
            if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path_str = exe_path.to_string_lossy().to_string();
                    let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = RegSetValueExW(
                        hkey,
                        APP_REG_NAME,
                        0,
                        REG_SZ,
                        Some(std::slice::from_raw_parts(
                            wide.as_ptr() as *const u8,
                            wide.len() * 2,
                        )),
                    );
                }
            } else {
                let _ = RegDeleteValueW(hkey, APP_REG_NAME);
            }
            let _ = RegCloseKey(hkey);
        }
    }
}

// Cria um ícone moderno de vidro translúcido com 4 quadrantes coloridos
unsafe fn create_glass_tray_icon() -> HICON {
    let size: usize = 32;
    let mut bgra = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;

            // Borda externa com margem de 2px (quadrado de 28x28 dentro de 32x32)
            if x < 2 || x >= 30 || y < 2 || y >= 30 {
                // Totalmente transparente fora do container
                bgra[idx] = 0;
                bgra[idx + 1] = 0;
                bgra[idx + 2] = 0;
                bgra[idx + 3] = 0;
                continue;
            }

            // Divisórias das 4 zonas (cruz central no x=15..16 ou y=15..16)
            if x == 15 || x == 16 || y == 15 || y == 16 {
                // Espaçamento de vidro translúcido sutil
                bgra[idx] = 40;
                bgra[idx + 1] = 40;
                bgra[idx + 2] = 40;
                bgra[idx + 3] = 180;
                continue;
            }

            // Borda de vidro luminosa externa
            if x == 2 || x == 29 || y == 2 || y == 29 {
                bgra[idx] = 255;
                bgra[idx + 1] = 255;
                bgra[idx + 2] = 255;
                bgra[idx + 3] = 200;
                continue;
            }

            // 4 Zonas com efeito de vidro vibrante (Blue, Emerald, Amber, Coral):
            let is_left = x < 15;
            let is_top = y < 15;

            let (r, g, b): (u8, u8, u8) = match (is_left, is_top) {
                (true, true) => (79, 140, 255),   // Top-Left: Azul Elétrico
                (false, true) => (46, 204, 113),  // Top-Right: Esmeralda
                (true, false) => (241, 196, 15),  // Bottom-Left: Âmbar/Amarelo
                (false, false) => (235, 77, 75),  // Bottom-Right: Coral/Vermelho
            };

            // Efeito de brilho de vidro na parte superior
            let gloss: u8 = if is_top && (y <= 6) { 50 } else { 0 };

            // BGRA layout para DIBSection / Bitmap do Win32
            bgra[idx] = b.saturating_add(gloss);
            bgra[idx + 1] = g.saturating_add(gloss);
            bgra[idx + 2] = r.saturating_add(gloss);
            bgra[idx + 3] = 230; // 90% opaco / efeito vidro
        }
    }

    let hdc_screen: HDC = GetDC(HWND::default());
    let mut bmi = std::mem::zeroed::<windows::Win32::Graphics::Gdi::BITMAPINFO>();
    bmi.bmiHeader.biSize = std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = size as i32;
    bmi.bmiHeader.biHeight = -(size as i32); // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = windows::Win32::Graphics::Gdi::BI_RGB.0;

    let mut bits_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let hbm_color: HBITMAP = windows::Win32::Graphics::Gdi::CreateDIBSection(
        hdc_screen,
        &bmi,
        windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
        &mut bits_ptr,
        windows::Win32::Foundation::HANDLE::default(),
        0,
    )
    .unwrap_or_default();

    if !bits_ptr.is_null() {
        std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits_ptr as *mut u8, bgra.len());
    }

    let hbm_mask: HBITMAP = CreateBitmap(size as i32, size as i32, 1, 1, None);
    let _ = ReleaseDC(HWND::default(), hdc_screen);

    let mut icon_info = ICONINFO {
        fIcon: windows::Win32::Foundation::BOOL(1),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = CreateIconIndirect(&mut icon_info).unwrap_or(HICON::default());

    let _ = DeleteObject(hbm_color);
    let _ = DeleteObject(hbm_mask);

    hicon
}

fn vk_to_char(vk: u32) -> Option<char> {
    match vk {
        v if v == VK_0.0 as u32 => Some('0'),
        v if v == VK_1.0 as u32 => Some('1'),
        v if v == VK_2.0 as u32 => Some('2'),
        v if v == VK_3.0 as u32 => Some('3'),
        v if v == VK_4.0 as u32 => Some('4'),
        v if v == VK_5.0 as u32 => Some('5'),
        v if v == VK_6.0 as u32 => Some('6'),
        v if v == VK_7.0 as u32 => Some('7'),
        v if v == VK_8.0 as u32 => Some('8'),
        v if v == VK_9.0 as u32 => Some('9'),
        v if v == VK_A.0 as u32 => Some('A'),
        v if v == VK_B.0 as u32 => Some('B'),
        v if v == VK_C.0 as u32 => Some('C'),
        v if v == VK_D.0 as u32 => Some('D'),
        v if v == VK_E.0 as u32 => Some('E'),
        v if v == VK_F.0 as u32 => Some('F'),
        v if v == VK_G.0 as u32 => Some('G'),
        v if v == VK_H.0 as u32 => Some('H'),
        v if v == VK_I.0 as u32 => Some('I'),
        v if v == VK_J.0 as u32 => Some('J'),
        v if v == VK_K.0 as u32 => Some('K'),
        v if v == VK_L.0 as u32 => Some('L'),
        v if v == VK_M.0 as u32 => Some('M'),
        v if v == VK_N.0 as u32 => Some('N'),
        v if v == VK_O.0 as u32 => Some('O'),
        v if v == VK_P.0 as u32 => Some('P'),
        v if v == VK_Q.0 as u32 => Some('Q'),
        v if v == VK_R.0 as u32 => Some('R'),
        v if v == VK_S.0 as u32 => Some('S'),
        v if v == VK_T.0 as u32 => Some('T'),
        v if v == VK_U.0 as u32 => Some('U'),
        v if v == VK_V.0 as u32 => Some('V'),
        v if v == VK_W.0 as u32 => Some('W'),
        v if v == VK_X.0 as u32 => Some('X'),
        v if v == VK_Y.0 as u32 => Some('Y'),
        v if v == VK_Z.0 as u32 => Some('Z'),
        _ => None,
    }
}

fn tile_active_window_to_zone(zone: &ZoneConfig, gap: i32) {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return;
        }

        let monitor: HMONITOR = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: RECT::default(),
            rcWork: RECT::default(),
            dwFlags: 0,
        };

        if !GetMonitorInfoW(monitor, &mut mi).as_bool() {
            return;
        }

        let work_left = mi.rcWork.left;
        let work_top = mi.rcWork.top;
        let work_w = mi.rcWork.right - mi.rcWork.left;
        let work_h = mi.rcWork.bottom - mi.rcWork.top;

        let left = work_left + (zone.x * work_w as f32).round() as i32;
        let top = work_top + (zone.y * work_h as f32).round() as i32;
        let right = work_left + ((zone.x + zone.w) * work_w as f32).round() as i32;
        let bottom = work_top + ((zone.y + zone.h) * work_h as f32).round() as i32;

        let x = left + gap;
        let y = top + gap;
        let w = ((right - left) - (gap * 2)).max(100);
        let h = ((bottom - top) - (gap * 2)).max(100);

        let _ = ShowWindow(hwnd, SW_RESTORE);

        let _ = SetWindowPos(
            hwnd,
            HWND::default(),
            x,
            y,
            w,
            h,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

fn sync_layout_ui(
    app: &AppWindow,
    config: &ConfigFile,
    zone_model: &Rc<VecModel<ZoneData>>,
    layout_model: &Rc<VecModel<LayoutItem>>,
) {
    let active_id = &config.active_layout_id;
    let current_layout = config
        .layouts
        .get(active_id)
        .cloned()
        .unwrap_or_else(|| config.layouts.values().next().cloned().unwrap());

    app.set_layout_name(current_layout.name.clone().into());

    let mut sorted_keys: Vec<_> = config.layouts.keys().cloned().collect();
    sorted_keys.sort();

    while layout_model.row_count() > 0 {
        layout_model.remove(0);
    }
    for k in sorted_keys {
        if let Some(lay) = config.layouts.get(&k) {
            layout_model.push(LayoutItem {
                id: lay.id.clone().into(),
                name: lay.name.clone().into(),
                is_active: lay.id == *active_id,
            });
        }
    }

    while zone_model.row_count() > 0 {
        zone_model.remove(0);
    }
    for z in &current_layout.zones {
        zone_model.push(ZoneData {
            id: z.id,
            key: z.key.clone().into(),
            title: z.title.clone().into(),
            x: z.x,
            y: z.y,
            w: z.w,
            h: z.h,
            color: get_color_for_id(z.id),
        });
    }

    app.set_selected_id(-1);
    app.set_selected_key("".into());
}

fn commit_current_zones(config: &mut ConfigFile, zone_model: &Rc<VecModel<ZoneData>>) {
    let active_id = config.active_layout_id.clone();
    let mut zones = Vec::new();
    for i in 0..zone_model.row_count() {
        let z = zone_model.row_data(i).unwrap();
        zones.push(ZoneConfig {
            id: z.id,
            key: z.key.to_string(),
            title: z.title.to_string(),
            x: z.x,
            y: z.y,
            w: z.w,
            h: z.h,
            color: None,
        });
    }
    if let Some(layout) = config.layouts.get_mut(&active_id) {
        layout.zones = zones;
    }
}

struct DaemonContext {
    hwnd: HWND,
    config: Arc<Mutex<ConfigFile>>,
    layout_id_map: Vec<String>,
}

static mut GLOBAL_DAEMON: Option<*mut DaemonContext> = None;
static mut GLOBAL_HOOK: Option<HHOOK> = None;

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if n_code >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kbd = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let win_pressed = (GetKeyState(VK_LWIN.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_RWIN.0 as i32) as u16 & 0x8000 != 0);
        let alt_pressed = (GetKeyState(VK_LMENU.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_RMENU.0 as i32) as u16 & 0x8000 != 0);
        let ctrl_pressed = (GetKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000 != 0)
            || (GetKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000 != 0);

        if win_pressed && !ctrl_pressed {
            // Win + Alt + T: Abrir o Editor Visual
            if alt_pressed && (kbd.vkCode == VK_T.0 as u32) {
                if let Some(daemon_ptr) = GLOBAL_DAEMON {
                    let daemon = &*daemon_ptr;
                    let _ = PostMessageW(daemon.hwnd, WM_APP_OPEN_EDITOR, WPARAM(0), LPARAM(0));
                    return LRESULT(1);
                }
            }

            // Win + [Key]: Ativar Zona
            if !alt_pressed {
                if let Some(ch) = vk_to_char(kbd.vkCode) {
                    if let Some(daemon_ptr) = GLOBAL_DAEMON {
                        let daemon = &*daemon_ptr;
                        let cfg = daemon.config.lock().unwrap();
                        if let Some(layout) = cfg.layouts.get(&cfg.active_layout_id) {
                            let ch_str = ch.to_string();
                            let target_idx = layout.zones.iter().position(|z| {
                                z.key.eq_ignore_ascii_case(&ch_str)
                            });
                            if let Some(idx) = target_idx {
                                drop(cfg);
                                let _ = PostMessageW(
                                    daemon.hwnd,
                                    WM_APP_TILE_ZONE,
                                    WPARAM(idx),
                                    LPARAM(0),
                                );
                                return LRESULT(1);
                            }
                        }
                    }
                }
            }
        }
    }

    CallNextHookEx(GLOBAL_HOOK.unwrap_or_default(), n_code, wparam, lparam)
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let daemon = match GLOBAL_DAEMON {
        Some(ptr) => &mut *ptr,
        None => return DefWindowProcW(hwnd, msg, wparam, lparam),
    };

    match msg {
        WM_APP_TILE_ZONE => {
            let zone_idx = wparam.0;
            let cfg = daemon.config.lock().unwrap();
            if let Some(layout) = cfg.layouts.get(&cfg.active_layout_id) {
                if let Some(zone) = layout.zones.get(zone_idx) {
                    tile_active_window_to_zone(zone, cfg.gap);
                }
            }
            LRESULT(0)
        }
        WM_TRAYICON => {
            let event = (lparam.0 & 0xffff) as u32;
            if event == WM_RBUTTONUP || event == WM_LBUTTONDBLCLK {
                if event == WM_LBUTTONDBLCLK {
                    let _ = PostMessageW(hwnd, WM_APP_OPEN_EDITOR, WPARAM(0), LPARAM(0));
                } else {
                    show_tray_menu(hwnd, daemon);
                }
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd_id = wparam.0 as usize;
            if cmd_id == IDM_OPEN_EDITOR {
                let _ = PostMessageW(hwnd, WM_APP_OPEN_EDITOR, WPARAM(0), LPARAM(0));
            } else if cmd_id == IDM_AUTOSTART {
                let current = is_autostart_enabled();
                set_autostart(!current);
            } else if cmd_id == IDM_EXIT {
                PostQuitMessage(0);
            } else if cmd_id >= IDM_LAYOUT_BASE && cmd_id < IDM_LAYOUT_BASE + daemon.layout_id_map.len() {
                let target_idx = cmd_id - IDM_LAYOUT_BASE;
                if let Some(layout_id) = daemon.layout_id_map.get(target_idx) {
                    let mut cfg = daemon.config.lock().unwrap();
                    cfg.active_layout_id = layout_id.clone();
                    save_config(&cfg);
                }
            }
            LRESULT(0)
        }
        WM_APP_OPEN_EDITOR => {
            slint::invoke_from_event_loop(|| {}).ok();
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn show_tray_menu(hwnd: HWND, daemon: &mut DaemonContext) {
    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);

    let hmenu: HMENU = CreatePopupMenu().unwrap_or_default();
    if hmenu.0 == std::ptr::null_mut() {
        return;
    }

    let cfg = daemon.config.lock().unwrap();
    let mut layouts: Vec<_> = cfg.layouts.values().cloned().collect();
    layouts.sort_by(|a, b| a.name.cmp(&b.name));
    daemon.layout_id_map.clear();

    let mut pos = 0;
    for (i, lay) in layouts.iter().enumerate() {
        daemon.layout_id_map.push(lay.id.clone());
        let mut flags = MF_STRING;
        if lay.id == cfg.active_layout_id {
            flags |= MF_CHECKED;
        } else {
            flags |= MF_UNCHECKED;
        }

        let name_wide: Vec<u16> = format!("Layout: {}", lay.name)
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let _ = InsertMenuW(
            hmenu,
            pos,
            flags | MF_BYPOSITION,
            IDM_LAYOUT_BASE + i,
            windows::core::PCWSTR(name_wide.as_ptr()),
        );
        pos += 1;
    }
    drop(cfg);

    let _ = InsertMenuW(
        hmenu,
        pos,
        MF_SEPARATOR | MF_BYPOSITION,
        0,
        windows::core::PCWSTR::null(),
    );
    pos += 1;

    let editor_str: Vec<u16> = "Editar Zonas (Win + Alt + T)\0".encode_utf16().collect();
    let _ = InsertMenuW(
        hmenu,
        pos,
        MF_STRING | MF_BYPOSITION,
        IDM_OPEN_EDITOR,
        windows::core::PCWSTR(editor_str.as_ptr()),
    );
    pos += 1;

    // Opção "Iniciar com o Windows"
    let autostart_flags = if is_autostart_enabled() {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING | MF_UNCHECKED
    };
    let autostart_str: Vec<u16> = "Iniciar com o Windows\0".encode_utf16().collect();
    let _ = InsertMenuW(
        hmenu,
        pos,
        autostart_flags | MF_BYPOSITION,
        IDM_AUTOSTART,
        windows::core::PCWSTR(autostart_str.as_ptr()),
    );
    pos += 1;

    let _ = InsertMenuW(
        hmenu,
        pos,
        MF_SEPARATOR | MF_BYPOSITION,
        0,
        windows::core::PCWSTR::null(),
    );
    pos += 1;

    let exit_str: Vec<u16> = "Sair do Marzone\0".encode_utf16().collect();
    let _ = InsertMenuW(
        hmenu,
        pos,
        MF_STRING | MF_BYPOSITION,
        IDM_EXIT,
        windows::core::PCWSTR(exit_str.as_ptr()),
    );

    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(
        hmenu,
        TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(hmenu);
}

fn create_tray_icon(hwnd: HWND, icon: HICON) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = icon;

    let tip = "Marzone Tiling Manager\0";
    for (i, c) in tip.encode_utf16().enumerate() {
        if i < nid.szTip.len() {
            nid.szTip[i] = c;
        }
    }

    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);
    }
    nid
}

fn main() -> Result<(), slint::PlatformError> {
    let shared_config = Arc::new(Mutex::new(load_config()));
    let app = AppWindow::new()?;
    let weak_app = app.as_weak();

    let zone_model: Rc<VecModel<ZoneData>> = Rc::new(VecModel::default());
    let layout_model: Rc<VecModel<LayoutItem>> = Rc::new(VecModel::default());

    app.set_zones(ModelRc::from(zone_model.clone()));
    app.set_layouts_list(ModelRc::from(layout_model.clone()));
    app.set_gap_val(shared_config.lock().unwrap().gap);

    {
        let cfg = shared_config.lock().unwrap();
        sync_layout_ui(&app, &cfg, &zone_model, &layout_model);
    }

    let cfg_gap = shared_config.clone();
    app.on_gap_changed(move |new_gap| {
        cfg_gap.lock().unwrap().gap = new_gap;
    });

    let app_weak_sel = app.as_weak();
    let cfg_sel = shared_config.clone();
    let zm_sel = zone_model.clone();
    let lm_sel = layout_model.clone();
    app.on_select_layout(move |lid| {
        let mut cfg = cfg_sel.lock().unwrap();
        commit_current_zones(&mut cfg, &zm_sel);
        let id_str = lid.to_string();
        if cfg.layouts.contains_key(&id_str) {
            cfg.active_layout_id = id_str;
            if let Some(a) = app_weak_sel.upgrade() {
                sync_layout_ui(&a, &cfg, &zm_sel, &lm_sel);
            }
        }
    });

    let app_weak_new = app.as_weak();
    let cfg_new = shared_config.clone();
    let zm_new = zone_model.clone();
    let lm_new = layout_model.clone();
    app.on_new_layout(move || {
        let mut cfg = cfg_new.lock().unwrap();
        commit_current_zones(&mut cfg, &zm_new);

        let count = cfg.layouts.len() + 1;
        let new_id = format!(
            "custom-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let new_name = format!("Layout {}", count);

        cfg.layouts.insert(
            new_id.clone(),
            LayoutConfig {
                id: new_id.clone(),
                name: new_name,
                zones: vec![
                    ZoneConfig {
                        id: 1,
                        key: "1".into(),
                        title: "Zona 1".into(),
                        x: 0.0,
                        y: 0.0,
                        w: 0.5,
                        h: 1.0,
                        color: None,
                    },
                    ZoneConfig {
                        id: 2,
                        key: "2".into(),
                        title: "Zona 2".into(),
                        x: 0.5,
                        y: 0.0,
                        w: 0.5,
                        h: 1.0,
                        color: None,
                    },
                ],
            },
        );

        cfg.active_layout_id = new_id;
        if let Some(a) = app_weak_new.upgrade() {
            sync_layout_ui(&a, &cfg, &zm_new, &lm_new);
        }
    });

    let app_weak_del = app.as_weak();
    let cfg_del = shared_config.clone();
    let zm_del = zone_model.clone();
    let lm_del = layout_model.clone();
    app.on_delete_layout(move |lid| {
        let mut cfg = cfg_del.lock().unwrap();
        if cfg.layouts.len() <= 1 {
            return;
        }
        let id_str = lid.to_string();
        cfg.layouts.remove(&id_str);

        if cfg.active_layout_id == id_str {
            cfg.active_layout_id = cfg.layouts.keys().next().cloned().unwrap();
        }

        if let Some(a) = app_weak_del.upgrade() {
            sync_layout_ui(&a, &cfg, &zm_del, &lm_del);
        }
    });

    let app_weak_ren = app.as_weak();
    let cfg_ren = shared_config.clone();
    let zm_ren = zone_model.clone();
    let lm_ren = layout_model.clone();
    app.on_rename_layout(move |new_name| {
        let name_str = new_name.trim().to_string();
        if !name_str.is_empty() {
            let mut cfg = cfg_ren.lock().unwrap();
            let active_id = cfg.active_layout_id.clone();
            if let Some(lay) = cfg.layouts.get_mut(&active_id) {
                lay.name = name_str;
            }
            if let Some(a) = app_weak_ren.upgrade() {
                sync_layout_ui(&a, &cfg, &zm_ren, &lm_ren);
            }
        }
    });

    let zm = zone_model.clone();
    app.on_zone_moved(move |id, dx, dy, cw, ch| {
        let cw_f = cw;
        let ch_f = ch;
        for i in 0..zm.row_count() {
            let mut z = zm.row_data(i).unwrap();
            if z.id == id {
                let nx = (z.x + (dx / cw_f)).clamp(0.0, 1.0 - z.w);
                let ny = (z.y + (dy / ch_f)).clamp(0.0, 1.0 - z.h);
                z.x = snap(nx);
                z.y = snap(ny);
                zm.set_row_data(i, z);
                break;
            }
        }
    });

    let zr = zone_model.clone();
    app.on_zone_resized(move |id, dx, dy, cw, ch| {
        let cw_f = cw;
        let ch_f = ch;
        for i in 0..zr.row_count() {
            let mut z = zr.row_data(i).unwrap();
            if z.id == id {
                let nw = (z.w + (dx / cw_f)).clamp(0.1, 1.0 - z.x);
                let nh = (z.h + (dy / ch_f)).clamp(0.1, 1.0 - z.y);
                z.w = snap(nw);
                z.h = snap(nh);
                zr.set_row_data(i, z);
                break;
            }
        }
    });

    let zkey = zone_model.clone();
    let app_weak_key = app.as_weak();
    app.on_select_key(move |new_key| {
        if let Some(a) = app_weak_key.upgrade() {
            let sel = a.get_selected_id();
            if sel != -1 {
                for i in 0..zkey.row_count() {
                    let mut z = zkey.row_data(i).unwrap();
                    if z.id == sel {
                        z.key = new_key.clone();
                        zkey.set_row_data(i, z);
                        a.set_selected_key(new_key);
                        break;
                    }
                }
            }
        }
    });

    let za = zone_model.clone();
    let app_weak_add = app.as_weak();
    app.on_add_zone(move || {
        let mut max_id = 0;
        for i in 0..za.row_count() {
            let z = za.row_data(i).unwrap();
            if z.id > max_id {
                max_id = z.id;
            }
        }
        let next_id = max_id + 1;
        let next_key = format!("{}", (za.row_count() % 9) + 1);
        let new_z = ZoneData {
            id: next_id,
            key: next_key.into(),
            title: "Nova Zona".into(),
            x: 0.1,
            y: 0.1,
            w: 0.35,
            h: 0.35,
            color: get_color_for_id(next_id),
        };
        za.push(new_z.clone());
        if let Some(a) = app_weak_add.upgrade() {
            a.set_selected_id(new_z.id);
            a.set_selected_key(new_z.key);
        }
    });

    let zdel = zone_model.clone();
    let app_weak_rem = app.as_weak();
    app.on_remove_zone(move || {
        if let Some(a) = app_weak_rem.upgrade() {
            let sel = a.get_selected_id();
            for i in 0..zdel.row_count() {
                if zdel.row_data(i).unwrap().id == sel {
                    zdel.remove(i);
                    a.set_selected_id(-1);
                    break;
                }
            }
        }
    });

    let zs = zone_model.clone();
    let cfg_save = shared_config.clone();
    let app_weak_save = app.as_weak();
    app.on_save_and_apply(move || {
        let mut cfg = cfg_save.lock().unwrap();
        let active_id = cfg.active_layout_id.clone();

        let mut zones = Vec::new();
        for i in 0..zs.row_count() {
            let z = zs.row_data(i).unwrap();
            zones.push(ZoneConfig {
                id: z.id,
                key: z.key.to_string(),
                title: z.title.to_string(),
                x: z.x,
                y: z.y,
                w: z.w,
                h: z.h,
                color: None,
            });
        }

        if let Some(layout) = cfg.layouts.get_mut(&active_id) {
            layout.zones = zones;
        }

        if let Some(a) = app_weak_save.upgrade() {
            cfg.gap = a.get_gap_val();
            let _ = a.hide();
        }

        save_config(&cfg);
        println!("Configurações de layout salvas com sucesso!");
    });

    let thread_cfg = shared_config.clone();
    let slint_weak = weak_app.clone();
    let (tx_ready, rx_ready) = std::sync::mpsc::channel();

    thread::spawn(move || unsafe {
        let class_name = w!("MarzoneDaemonClass");
        let icon = create_glass_tray_icon();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(std::ptr::null_mut()),
            hIcon: icon,
            hCursor: windows::Win32::UI::WindowsAndMessaging::HCURSOR(std::ptr::null_mut()),
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(std::ptr::null_mut()),
            lpszMenuName: windows::core::PCWSTR::null(),
            lpszClassName: class_name,
            hIconSm: icon,
        };

        let _ = RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
            class_name,
            w!("Marzone Daemon"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        );

        if hwnd.is_err() {
            let _ = tx_ready.send(false);
            return;
        }
        let hwnd = hwnd.unwrap();

        let mut daemon = Box::new(DaemonContext {
            hwnd,
            config: thread_cfg,
            layout_id_map: Vec::new(),
        });

        GLOBAL_DAEMON = Some(&mut *daemon as *mut _);

        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            HINSTANCE(std::ptr::null_mut()),
            0,
        );
        GLOBAL_HOOK = hook.ok();

        let nid = create_tray_icon(hwnd, icon);
        let _ = tx_ready.send(true);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            if msg.message == WM_APP_OPEN_EDITOR {
                let slint_ui = slint_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(a) = slint_ui.upgrade() {
                        let _ = a.show();
                    }
                });
            } else {
                let _ = DispatchMessageW(&msg);
            }
        }

        if let Some(h) = GLOBAL_HOOK {
            let _ = UnhookWindowsHookEx(h);
        }
        let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        let _ = DestroyIcon(icon);
        let _ = DestroyWindow(hwnd);
    });

    let _ = rx_ready.recv().ok();
    println!("Marzone Windows iniciado com sucesso na bandeja do sistema.");

    // Mantém a janela oculta inicialmente
    let _ = app.hide();

    // Quando o usuário fecha o editor (X), apenas oculta a janela em vez de encerrar o daemon
    let app_weak_close = weak_app.clone();
    app.window().on_close_requested(move || {
        if let Some(a) = app_weak_close.upgrade() {
            let _ = a.hide();
        }
        slint::CloseRequestResponse::KeepWindowShown
    });

    // Mantém o loop de eventos ativo para responder ao tray e aos atalhos
    loop {
        std::thread::park();
    }
}
