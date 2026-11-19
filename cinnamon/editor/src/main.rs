use serde::{Deserialize, Serialize};
use slint::{Color, Model, ModelRc, RgbaColor, VecModel};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();

const VIBRANT_PALETTE: &[(u8, u8, u8)] = &[
    (79, 140, 255),  // Azul elétrico
    (46, 204, 113),  // Esmeralda / Verde
    (241, 196, 15),  // Amarelo vibrante
    (230, 126, 34),  // Laranja
    (231, 76, 60),   // Vermelho vivo
    (155, 89, 182),  // Roxo
    (26, 188, 156),  // Turquesa
    (255, 107, 129), // Rosa coral
    (52, 152, 219),  // Azul celeste
    (0, 210, 211),   // Ciano
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
    let mut p = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    p.push(".config");
    p.push("marzone");
    p.push("layouts.json");
    p
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
                parsed.active_layout_id = parsed.layouts.keys().next().cloned().unwrap_or_else(|| "focus-code".to_string());
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

    // Atualiza lista de layouts
    let mut sorted_keys: Vec<_> = config.layouts.keys().cloned().collect();
    sorted_keys.sort();

    // Limpa e repovoa layout_model
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

    // Limpa e repovoa zone_model
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

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let config = Rc::new(std::cell::RefCell::new(load_config()));

    let zone_model: Rc<VecModel<ZoneData>> = Rc::new(VecModel::default());
    let layout_model: Rc<VecModel<LayoutItem>> = Rc::new(VecModel::default());

    app.set_zones(ModelRc::from(zone_model.clone()));
    app.set_layouts_list(ModelRc::from(layout_model.clone()));
    app.set_gap_val(config.borrow().gap);

    sync_layout_ui(&app, &config.borrow(), &zone_model, &layout_model);

    // Callback de mudança de gap/padding
    let cfg_gap = config.clone();
    app.on_gap_changed(move |new_gap| {
        cfg_gap.borrow_mut().gap = new_gap;
    });

    // Callback de alternar layout
    let app_weak_sel = app.as_weak();
    let cfg_sel = config.clone();
    let zm_sel = zone_model.clone();
    let lm_sel = layout_model.clone();
    app.on_select_layout(move |lid| {
        let mut cfg = cfg_sel.borrow_mut();
        commit_current_zones(&mut cfg, &zm_sel);
        let id_str = lid.to_string();
        if cfg.layouts.contains_key(&id_str) {
            cfg.active_layout_id = id_str;
            if let Some(a) = app_weak_sel.upgrade() {
                sync_layout_ui(&a, &cfg, &zm_sel, &lm_sel);
            }
        }
    });

    // Callback de criar novo layout
    let app_weak_new = app.as_weak();
    let cfg_new = config.clone();
    let zm_new = zone_model.clone();
    let lm_new = layout_model.clone();
    app.on_new_layout(move || {
        let mut cfg = cfg_new.borrow_mut();
        commit_current_zones(&mut cfg, &zm_new);

        let count = cfg.layouts.len() + 1;
        let new_id = format!("custom-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
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

    // Callback de deletar layout
    let app_weak_del = app.as_weak();
    let cfg_del = config.clone();
    let zm_del = zone_model.clone();
    let lm_del = layout_model.clone();
    app.on_delete_layout(move |lid| {
        let mut cfg = cfg_del.borrow_mut();
        if cfg.layouts.len() <= 1 {
            return; // Não apaga o último
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

    // Callback de renomear layout
    let app_weak_ren = app.as_weak();
    let cfg_ren = config.clone();
    let zm_ren = zone_model.clone();
    let lm_ren = layout_model.clone();
    app.on_rename_layout(move |new_name| {
        let name_str = new_name.trim().to_string();
        if !name_str.is_empty() {
            let mut cfg = cfg_ren.borrow_mut();
            let active_id = cfg.active_layout_id.clone();
            if let Some(lay) = cfg.layouts.get_mut(&active_id) {
                lay.name = name_str;
            }
            if let Some(a) = app_weak_ren.upgrade() {
                sync_layout_ui(&a, &cfg, &zm_ren, &lm_ren);
            }
        }
    });

    // Callback de mover zona
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

    // Callback de redimensionar zona
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

    // Callback de selecionar tecla para a zona ativa
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

    // Callback de adicionar zona com cor vibrante aleatória/distinta
    let za = zone_model.clone();
    let app_weak = app.as_weak();
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
        if let Some(a) = app_weak.upgrade() {
            a.set_selected_id(new_z.id);
            a.set_selected_key(new_z.key);
        }
    });

    // Callback de remover zona
    let zdel = zone_model.clone();
    let app_weak2 = app.as_weak();
    app.on_remove_zone(move || {
        if let Some(a) = app_weak2.upgrade() {
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

    // Salvar e aplicar
    let zs = zone_model.clone();
    let cfg_save = config.clone();
    let app_weak3 = app.as_weak();
    app.on_save_and_apply(move || {
        let mut cfg = cfg_save.borrow_mut();
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

        if let Some(a) = app_weak3.upgrade() {
            cfg.gap = a.get_gap_val();
        }

        save_config(&cfg);
        println!("Configurações de layout salvas com sucesso!");

        if let Some(a) = app_weak3.upgrade() {
            let _ = a.hide();
        }
    });

    app.run()
}
