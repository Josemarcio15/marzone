// applet.js — Cinnamon Panel Applet & Window Tiling Controller
const Applet = imports.ui.applet;
const Main = imports.ui.main;
const PopupMenu = imports.ui.popupMenu;
const St = imports.gi.St;
const GLib = imports.gi.GLib;
const Gio = imports.gi.Gio;
const Meta = imports.gi.Meta;
const Util = imports.misc.util;

// Importa o modelo
let TilingModel;
try {
  let appletDir = imports.ui.appletManager.appletMeta['marzone@marcio']?.path;
  if (!appletDir) {
    appletDir = GLib.build_filenamev([GLib.get_home_dir(), '.local', 'share', 'cinnamon', 'applets', 'marzone@marcio']);
  }
  imports.searchPath.unshift(appletDir);
  TilingModel = imports.tilingModel;
} catch (e) {
  global.log('Marzone: Falha ao carregar tilingModel: ' + e);
}

class MarzoneApplet extends Applet.IconApplet {
  constructor(metadata, orientation, panel_height, instance_id) {
    super(orientation, panel_height, instance_id);
    this.metadata = metadata;
    this.instance_id = instance_id;

    this.set_applet_icon_name('preferences-system-windows');
    this.set_applet_tooltip(_('Marzone Window Tiling'));

    this.currentLayoutId = 'focus-code';
    this.gap = 6;
    this.keybindings = [];
    this.layouts = {};

    if (!TilingModel && typeof require !== 'undefined') {
      try { TilingModel = require('./tilingModel'); } catch (e) {}
    }

    this.menuManager = new PopupMenu.PopupMenuManager(this);
    this.menu = new Applet.AppletPopupMenu(this, orientation);
    this.menuManager.addMenu(this.menu);

    this._reloadLayouts();
    this._monitorConfigFile();
  }

  _monitorConfigFile() {
    try {
      let configPath = TilingModel.getConfigFile();
      let file = Gio.File.new_for_path(configPath);
      this._monitor = file.monitor_file(Gio.FileMonitorFlags.NONE, null);
      this._monitor.connect('changed', (monitor, file, other_file, event_type) => {
        if (event_type === Gio.FileMonitorEvent.CHANGES_DONE_HINT || event_type === Gio.FileMonitorEvent.CREATED) {
          this._reloadLayouts();
          let activeName = this.layouts[this.currentLayoutId]?.name || this.currentLayoutId;
          Main.notify(_('Marzone'), _(`Layout atualizado: ${activeName}`));
        }
      });
    } catch (e) {
      global.log('Marzone: Falha ao iniciar monitor do arquivo de configuração: ' + e);
    }
  }

  _reloadLayouts() {
    let result = TilingModel.loadAllLayouts();
    this.layouts = result.layouts || {};
    this.gap = (typeof result.gap === 'number') ? result.gap : 6;

    if (result.activeLayoutId && this.layouts[result.activeLayoutId]) {
      this.currentLayoutId = result.activeLayoutId;
    } else if (!this.layouts[this.currentLayoutId]) {
      this.currentLayoutId = Object.keys(this.layouts)[0] || 'focus-code';
    }
    this._buildMenu();
    this._registerHotkeys();
  }

  _buildMenu() {
    this.menu.removeAll();

    let header = new PopupMenu.PopupMenuItem(_('Layout Ativo:'), { reactive: false });
    this.menu.addMenuItem(header);

    for (let id in this.layouts) {
      let layout = this.layouts[id];
      let item = new PopupMenu.PopupMenuItem(layout.name);
      if (id === this.currentLayoutId) {
        item.setShowDot(true);
      }
      item.connect('activate', () => {
        this._switchLayout(id);
      });
      this.menu.addMenuItem(item);
    }

    this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

    let actionsHeader = new PopupMenu.PopupMenuItem(_('Atalhos: Super + [1..9]'), { reactive: false });
    this.menu.addMenuItem(actionsHeader);

    let curr = this.layouts[this.currentLayoutId];
    if (curr && curr.zones) {
      curr.zones.forEach(zone => {
        let label = `Zona [${zone.key}]: ${zone.title || 'Slot ' + zone.id}`;
        let zItem = new PopupMenu.PopupMenuItem(label);
        zItem.connect('activate', () => {
          this.tileFocusedWindow(zone.key);
        });
        this.menu.addMenuItem(zItem);
      });
    }

    this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

    // Seletor de Espaçamento (Gap)
    let gapSubMenu = new PopupMenu.PopupSubMenuMenuItem(_(`Espaçamento (Gap): ${this.gap}px`));
    [0, 2, 4, 6, 8, 10, 14, 20].forEach(g => {
      let label = (g === 0) ? _('0px (Sem espaço / Coladas)') : `${g}px`;
      let gItem = new PopupMenu.PopupMenuItem(label);
      if (this.gap === g) {
        gItem.setShowDot(true);
      }
      gItem.connect('activate', () => {
        this.gap = g;
        TilingModel.saveConfig(this.layouts, this.gap, this.currentLayoutId);
        Main.notify(_('Marzone'), _(`Espaçamento definido para: ${g}px`));
        this._buildMenu();
      });
      gapSubMenu.menu.addMenuItem(gItem);
    });
    this.menu.addMenuItem(gapSubMenu);

    this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

    // Botão para abrir o Editor Visual
    let editorItem = new PopupMenu.PopupIconMenuItem(_('Abrir Editor Visual de Zonas...'), 'document-edit', St.IconType.SYMBOLIC);
    editorItem.connect('activate', () => {
      this.openVisualEditor();
    });
    this.menu.addMenuItem(editorItem);
  }

  openVisualEditor() {
    let internalSlint = GLib.build_filenamev([this.metadata.path, 'marzone-editor']);
    let rootSlint = GLib.build_filenamev([GLib.get_home_dir(), 'git', 'marzone', 'target', 'release', 'marzone-editor']);
    let localSlint = GLib.build_filenamev([GLib.get_home_dir(), 'git', 'marzone', 'cinnamon', 'editor', 'target', 'release', 'marzone-editor']);

    if (GLib.file_test(internalSlint, GLib.FileTest.EXISTS)) {
      Util.spawnCommandLine(`"${internalSlint}"`);
    } else if (GLib.file_test(rootSlint, GLib.FileTest.EXISTS)) {
      Util.spawnCommandLine(`"${rootSlint}"`);
    } else if (GLib.file_test(localSlint, GLib.FileTest.EXISTS)) {
      Util.spawnCommandLine(`"${localSlint}"`);
    } else {
      Main.notify(_('Marzone'), _('Compile o editor executando cargo build --release -p marzone-editor'));
    }
  }

  _switchLayout(layoutId) {
    this.currentLayoutId = layoutId;
    let name = this.layouts[layoutId]?.name || layoutId;
    Main.notify(_('Marzone'), _(`Layout alterado para: ${name}`));
    this._buildMenu();
    this._registerHotkeys();
  }

  on_applet_clicked(event) {
    this._reloadLayouts();
    this.menu.toggle();
  }

  _unregisterHotkeys() {
    for (let binding of this.keybindings) {
      Main.keybindingManager.removeHotKey(binding);
    }
    this.keybindings = [];
  }

  _registerHotkeys() {
    this._unregisterHotkeys();

    let layout = this.layouts[this.currentLayoutId];
    if (!layout || !layout.zones) return;

    layout.zones.forEach(zone => {
      let id = `marzone-tile-${zone.key}`;
      let accel = `<Super>${zone.key}`;
      
      Main.keybindingManager.addHotKey(id, accel, () => {
        this.tileFocusedWindow(zone.key);
      });
      this.keybindings.push(id);
    });

    // Atalho global Super + Alt + T abre o Editor Visual direto
    let toggleId = 'marzone-toggle-editor';
    Main.keybindingManager.addHotKey(toggleId, '<Super><Alt>t', () => {
      this.openVisualEditor();
    });
    this.keybindings.push(toggleId);
  }

  tileFocusedWindow(zoneKey) {
    let win = global.display.get_focus_window();
    if (!win) return;

    let type = win.get_window_type();
    if (type === Meta.WindowType.DESKTOP || type === Meta.WindowType.DOCK) {
      return;
    }

    if (win.get_maximized()) {
      win.unmaximize(Meta.MaximizeFlags.BOTH);
    }

    let workArea = win.get_work_area_current_monitor();
    let layout = this.layouts[this.currentLayoutId];
    let zone = TilingModel.findZoneByKey(layout, zoneKey);
    if (!zone) return;

    let target = TilingModel.computeZoneRect(zone, workArea, this.gap);
    if (!target) return;

    win.move_resize_frame(true, target.x, target.y, target.width, target.height);
  }

  on_applet_removed_from_panel() {
    this._unregisterHotkeys();
  }
}

function main(metadata, orientation, panel_height, instance_id) {
  return new MarzoneApplet(metadata, orientation, panel_height, instance_id);
}
