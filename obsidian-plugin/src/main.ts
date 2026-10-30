import { Notice, Plugin } from 'obsidian';
import { launchCuesor } from './cuesor-locator';
import { CursorBadgeManager } from './cursor-badge-manager';
import { CursorColorManager } from './cursor-color-manager';
import { t } from './i18n';
import { ImeSensor } from './ime-sensor';
import { CuesorSettingTab } from './settings-tab';
import { StateStore } from './state-store';
import { CuesorPluginSettings, DEFAULT_SETTINGS, IndicatorMode } from './types';

export default class CuesorPlugin extends Plugin {
  public settings: CuesorPluginSettings = { ...DEFAULT_SETTINGS };
  public store!: StateStore;
  public colorManager!: CursorColorManager;
  public badgeManager!: CursorBadgeManager;
  public sensor!: ImeSensor;

  public async onload(): Promise<void> {
    await this.loadSettings();

    this.store = new StateStore(this.settings);
    this.colorManager = new CursorColorManager();
    this.badgeManager = new CursorBadgeManager(this.app);
    this.sensor = new ImeSensor(this.store);

    this.colorManager.initialize();
    this.badgeManager.initialize();

    // Subscribe visual renderers to state changes
    this.register(
      this.store.subscribe((snapshot) => {
        this.colorManager.render(snapshot);
        this.badgeManager.render(snapshot);
      })
    );

    // Initialize IME sensor
    await this.sensor.initialize();

    // Register editor and layout tracking events
    this.registerInterval(window.setInterval(() => this.badgeManager.updatePosition(), 80));
    this.registerDomEvent(document, 'scroll', () => this.badgeManager.updatePosition(), true);
    this.registerDomEvent(document, 'focusin', () => this.badgeManager.updatePosition());
    this.registerDomEvent(document, 'focusout', () => this.badgeManager.hide());
    this.registerDomEvent(window, 'blur', () => this.badgeManager.hide());
    this.registerDomEvent(window, 'resize', () => this.badgeManager.updatePosition());

    this.registerEvent(this.app.workspace.on('active-leaf-change', () => this.badgeManager.updatePosition()));
    this.registerEvent(this.app.workspace.on('editor-change', () => this.badgeManager.updatePosition()));

    // Register user commands
    this.addCommand({
      id: 'toggle-indicator-mode',
      name: t('command_toggle_indicator_mode'),
      callback: () => {
        const modes: IndicatorMode[] = ['both', 'color', 'badge', 'none'];
        const nextIdx = (modes.indexOf(this.settings.indicatorMode) + 1) % modes.length;
        this.settings.indicatorMode = modes[nextIdx];
        this.saveSettings();
        new Notice(t('notice_indicator_mode_changed', { mode: this.settings.indicatorMode }));
      },
    });

    this.addCommand({
      id: 'launch-cuesor',
      name: t('command_launch_cuesor'),
      callback: async () => {
        const snapshot = this.store.getSnapshot();
        if (snapshot.bridgeConnected) {
          new Notice(t('notice_already_connected'));
          return;
        }

        const path = snapshot.cuesorPath;
        if (!path) {
          this.showDisconnectedNotice();
          return;
        }

        new Notice(t('notice_launching_cuesor'));
        const result = await launchCuesor(path);
        if (result.success) {
          setTimeout(() => {
            this.sensor.reconnectBridge();
          }, 1500);
        } else {
          new Notice(t('notice_launch_failed', { error: result.error || t('error_unknown') }));
        }
      },
    });

    this.addCommand({
      id: 'reconnect-cuesor-bridge',
      name: t('command_reconnect_bridge'),
      callback: () => {
        this.sensor.reconnectBridge();
        setTimeout(() => {
          const snapshot = this.store.getSnapshot();
          if (snapshot.bridgeConnected) {
            new Notice(t('notice_reconnected_success'));
          } else {
            this.showDisconnectedNotice();
          }
        }, 800);
      },
    });

    this.addCommand({
      id: 'toggle-simulated-ime-mode',
      name: t('command_toggle_simulated_ime'),
      callback: () => {
        this.sensor.toggleManualMode();
      },
    });

    // Add settings tab
    this.addSettingTab(new CuesorSettingTab(this.app, this));
  }

  public showDisconnectedNotice(): void {
    const snapshot = this.store.getSnapshot();
    const path = snapshot.cuesorPath;

    const frag = document.createDocumentFragment();
    frag.appendText(t('notice_disconnected_header'));

    if (path) {
      const launchLink = frag.createEl('a', { text: t('notice_click_to_launch') });
      launchLink.style.textDecoration = 'underline';
      launchLink.style.cursor = 'pointer';
      launchLink.style.fontWeight = 'bold';
      launchLink.onclick = async (e) => {
        e.preventDefault();
        new Notice(t('notice_launching_cuesor'));
        const res = await launchCuesor(path);
        if (res.success) {
          setTimeout(() => this.sensor.reconnectBridge(), 1500);
        }
      };
    } else {
      frag.appendText(t('notice_disconnected_guide'));
    }

    new Notice(frag, 6000);
  }

  public onunload(): void {
    this.sensor.destroy();
    this.colorManager.destroy();
    this.badgeManager.destroy();
  }

  public async loadSettings(): Promise<void> {
    const loaded = (await this.loadData()) as any;
    this.settings = Object.assign({}, DEFAULT_SETTINGS, loaded);

    // Migration and defaults normalization
    if (!this.settings.colors) {
      this.settings.colors = { ...DEFAULT_SETTINGS.colors };
    }
    if (this.settings.colors.syncWithCuesor === undefined) {
      this.settings.colors.syncWithCuesor = true;
    }
    if (!this.settings.colors.zh) {
      this.settings.colors.zh = (loaded?.colors?.zhLight) || DEFAULT_SETTINGS.colors.zh;
    }
    if (!this.settings.colors.en) {
      this.settings.colors.en = (loaded?.colors?.enLight) || DEFAULT_SETTINGS.colors.en;
    }
    if (!this.settings.bridge) {
      this.settings.bridge = { ...DEFAULT_SETTINGS.bridge };
    }
    if (this.settings.bridge.cuesorPath === undefined) {
      this.settings.bridge.cuesorPath = '';
    }
  }

  public async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
    this.store.updateSettings(this.settings);
  }
}
