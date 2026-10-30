import { Notice, Plugin } from 'obsidian';
import { launchCuesor } from './cuesor-locator';
import { CursorBadgeManager } from './cursor-badge-manager';
import { CursorColorManager } from './cursor-color-manager';
import { t } from './i18n';
import { ImeSensor } from './ime-sensor';
import { CuesorSettingTab } from './settings-tab';
import { StateStore } from './state-store';
import { CuesorPluginSettings, DEFAULT_SETTINGS, IndicatorMode } from './types';

function cloneDefaults(): CuesorPluginSettings {
  return {
    indicatorMode: DEFAULT_SETTINGS.indicatorMode,
    colors: { ...DEFAULT_SETTINGS.colors },
    badge: { ...DEFAULT_SETTINGS.badge },
    bridge: { ...DEFAULT_SETTINGS.bridge },
  };
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : null;
}

function nonEmptyText(value: unknown): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : null;
}

function finiteNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function isIndicatorMode(value: unknown): value is IndicatorMode {
  return value === 'both' || value === 'color' || value === 'badge' || value === 'none';
}

export default class CuesorPlugin extends Plugin {
  public settings: CuesorPluginSettings = cloneDefaults();
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

    this.sensor.initialize();

    // Register editor and layout tracking events
    this.registerInterval(window.setInterval(() => this.badgeManager.scheduleUpdate(), 80));
    this.registerDomEvent(document, 'scroll', () => this.badgeManager.scheduleUpdate(), true);
    this.registerDomEvent(document, 'focusin', () => this.badgeManager.scheduleUpdate());
    this.registerDomEvent(document, 'focusout', () => this.badgeManager.hide());
    this.registerDomEvent(window, 'blur', () => this.badgeManager.hide());
    this.registerDomEvent(window, 'resize', () => this.badgeManager.scheduleUpdate());

    this.registerEvent(this.app.workspace.on('active-leaf-change', () => this.badgeManager.scheduleUpdate()));
    this.registerEvent(this.app.workspace.on('editor-change', () => this.badgeManager.scheduleUpdate()));

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
    const loaded = (await this.loadData()) as Record<string, unknown> | null;
    const settings = cloneDefaults();
    if (!loaded) {
      this.settings = settings;
      return;
    }

    if (isIndicatorMode(loaded.indicatorMode)) {
      settings.indicatorMode = loaded.indicatorMode;
    }

    const colors = asRecord(loaded.colors);
    if (colors) {
      if (typeof colors.syncWithCuesor === 'boolean') {
        settings.colors.syncWithCuesor = colors.syncWithCuesor;
      }
      settings.colors.zh =
        nonEmptyText(colors.zh) ?? nonEmptyText(colors.zhLight) ?? settings.colors.zh;
      settings.colors.en =
        nonEmptyText(colors.en) ?? nonEmptyText(colors.enLight) ?? settings.colors.en;
    }

    const badge = asRecord(loaded.badge);
    if (badge) {
      settings.badge.textZh = nonEmptyText(badge.textZh) ?? settings.badge.textZh;
      settings.badge.textEn = nonEmptyText(badge.textEn) ?? settings.badge.textEn;
      settings.badge.textUnknown = nonEmptyText(badge.textUnknown) ?? settings.badge.textUnknown;
      settings.badge.offsetX = finiteNumber(badge.offsetX, settings.badge.offsetX);
      settings.badge.offsetY = finiteNumber(badge.offsetY, settings.badge.offsetY);
    }

    const bridge = asRecord(loaded.bridge);
    if (bridge) {
      settings.bridge.cuesorPath = nonEmptyText(bridge.cuesorPath) ?? '';
    }

    this.settings = settings;
  }

  public async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
    this.store.updateSettings(this.settings);
  }
}
