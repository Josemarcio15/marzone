// tilingModel.js — Lógica de modelo, cálculo de zonas e leitura de layouts salvos
const GLib = imports.gi.GLib;
const Gio = imports.gi.Gio;

var AVAILABLE_KEYS = [
  '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
  'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P',
  'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
  'Z', 'X', 'C', 'V', 'B', 'N', 'M'
];

var DEFAULT_TILING_LAYOUTS = {
  'focus-code': {
    id: 'focus-code',
    name: 'Foco + Terminais',
    zones: [
      { id: 1, key: '1', title: 'Editor Principal (65%)', x: 0, y: 0, w: 0.65, h: 1 },
      { id: 2, key: '2', title: 'Terminal Sup (35%)', x: 0.65, y: 0, w: 0.35, h: 0.5 },
      { id: 3, key: '3', title: 'Terminal Inf (35%)', x: 0.65, y: 0.5, w: 0.35, h: 0.5 }
    ]
  },
  '3-columns': {
    id: '3-columns',
    name: '3 Colunas',
    zones: [
      { id: 1, key: '1', title: 'Esquerda (1/3)', x: 0, y: 0, w: 1 / 3, h: 1 },
      { id: 2, key: '2', title: 'Centro (1/3)', x: 1 / 3, y: 0, w: 1 / 3, h: 1 },
      { id: 3, key: '3', title: 'Direita (1/3)', x: 2 / 3, y: 0, w: 1 / 3, h: 1 }
    ]
  },
  'quarters': {
    id: 'quarters',
    name: '4 Quadrantes',
    zones: [
      { id: 7, key: '7', title: 'Sup. Esquerdo', x: 0, y: 0, w: 0.5, h: 0.5 },
      { id: 9, key: '9', title: 'Sup. Direito', x: 0.5, y: 0, w: 0.5, h: 0.5 },
      { id: 1, key: '1', title: 'Inf. Esquerdo', x: 0, y: 0.5, w: 0.5, h: 0.5 },
      { id: 3, key: '3', title: 'Inf. Direito', x: 0.5, y: 0.5, w: 0.5, h: 0.5 }
    ]
  },
  'halves': {
    id: 'halves',
    name: '2 Metades (Split)',
    zones: [
      { id: 1, key: '1', title: 'Esquerda', x: 0, y: 0, w: 0.5, h: 1 },
      { id: 2, key: '2', title: 'Direita', x: 0.5, y: 0, w: 0.5, h: 1 }
    ]
  }
};

function getConfigFile() {
  return GLib.build_filenamev([GLib.get_home_dir(), '.config', 'marzone', 'layouts.json']);
}

function loadAllLayouts() {
  let layouts = Object.assign({}, DEFAULT_TILING_LAYOUTS);
  let gap = 6;
  let activeLayoutId = null;
  try {
    let path = getConfigFile();
    let file = Gio.File.new_for_path(path);
    if (file.query_exists(null)) {
      let [ok, contents] = file.load_contents(null);
      if (ok) {
        let parsed = JSON.parse(new TextDecoder().decode(contents));
        if (parsed) {
          if (parsed.layouts) {
            Object.assign(layouts, parsed.layouts);
          }
          if (typeof parsed.gap === 'number') {
            gap = parsed.gap;
          }
          if (parsed.activeLayoutId && typeof parsed.activeLayoutId === 'string') {
            activeLayoutId = parsed.activeLayoutId;
          }
        }
      }
    }
  } catch (e) {
    global.log('Marzone: Falha ao carregar layouts customizados: ' + e);
  }
  return { layouts: layouts, gap: gap, activeLayoutId: activeLayoutId };
}

function saveConfig(layouts, gap, activeLayoutId) {
  try {
    let dirPath = GLib.build_filenamev([GLib.get_home_dir(), '.config', 'marzone']);
    GLib.mkdir_with_parents(dirPath, 0o755);
    let filePath = getConfigFile();
    let file = Gio.File.new_for_path(filePath);
    let data = {
      activeLayoutId: activeLayoutId || 'focus-code',
      gap: gap,
      layouts: layouts
    };
    file.replace_contents(JSON.stringify(data, null, 2), null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);
  } catch (e) {
    global.log('Marzone: Falha ao salvar configurações: ' + e);
  }
}

function computeZoneRect(zone, workArea, gap = 4) {
  if (!zone || !workArea) return null;

  let left = Math.round(workArea.x + zone.x * workArea.width);
  let top = Math.round(workArea.y + zone.y * workArea.height);
  let right = Math.round(workArea.x + (zone.x + zone.w) * workArea.width);
  let bottom = Math.round(workArea.y + (zone.y + zone.h) * workArea.height);

  // Se gap for 0, as janelas se tocam exatamente sem espaço vazio
  let x = left + gap;
  let y = top + gap;
  let w = Math.max(100, (right - left) - (gap * 2));
  let h = Math.max(100, (bottom - top) - (gap * 2));

  return {
    x: x,
    y: y,
    width: w,
    height: h
  };
}

function findZoneByKey(layout, key) {
  if (!layout || !Array.isArray(layout.zones) || !key) return null;
  let target = String(key).trim().toLowerCase();
  return layout.zones.find(z => String(z.key).toLowerCase() === target) || null;
}
