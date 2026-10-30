import { App, PluginSettingTab, Setting, Notice } from 'obsidian';
import { launchCuesor, readCuesorSettingsColors } from './cuesor-locator';
import { t } from './i18n';
import type CuesorPlugin from './main';
import { BRIDGE_PORT, IndicatorMode } from './types';

export class CuesorSettingTab extends PluginSettingTab {
  private readonly plugin: CuesorPlugin;
  private pathRefreshTimer: number | null = null;

  constructor(app: App, plugin: CuesorPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  public hide(): void {
    if (this.pathRefreshTimer !== null) {
      window.clearTimeout(this.pathRefreshTimer);
      this.pathRefreshTimer = null;
    }
    super.hide();
  }

  public display(): void {
    const { containerEl } = this;
    containerEl.empty();

    containerEl.createEl('h2', { text: t('settings_title') });

    this.renderStatusCard(containerEl);
    this.renderCoreSettings(containerEl);
    this.renderColorSettings(containerEl);
    this.renderBadgeSettings(containerEl);
    this.renderBridgeSettings(containerEl);
  }

  private renderStatusCard(containerEl: HTMLElement): void {
    const snapshot = this.plugin.store.getSnapshot();
    const isConnected = snapshot.bridgeConnected;
    const resolvedPath = snapshot.cuesorPath;

    const card = containerEl.createDiv({
      cls: `cuesor-status-card ${isConnected ? 'connected' : 'disconnected'}`,
    });

    const header = card.createDiv({ cls: 'cuesor-status-header' });
    header.createSpan({ cls: 'cuesor-status-dot' });
    header.createSpan({
      cls: 'cuesor-status-title',
      text: isConnected ? t('status_title_connected') : t('status_title_disconnected'),
    });

    const desc = card.createDiv({ cls: 'cuesor-status-desc' });
    if (isConnected) {
      desc.setText(t('status_desc_connected', { port: BRIDGE_PORT }));
    } else {
      desc.setText(t('status_desc_disconnected', { port: BRIDGE_PORT }));

      const actions = card.createDiv({ cls: 'cuesor-status-actions' });
      const pathInfo = actions.createDiv({ cls: 'cuesor-detected-path' });

      if (resolvedPath) {
        pathInfo.createSpan({ text: t('status_detected_path') });
        pathInfo.createEl('code', { text: resolvedPath });

        const btnGroup = actions.createDiv({ cls: 'cuesor-btn-group' });
        const launchBtn = btnGroup.createEl('button', {
          text: t('status_btn_launch'),
          cls: 'mod-cta',
        });
        launchBtn.onclick = async () => {
          new Notice(t('notice_launching_cuesor'));
          const result = await launchCuesor(resolvedPath);
          if (result.success) {
            setTimeout(async () => {
              this.plugin.sensor.reconnectBridge();
              setTimeout(() => this.display(), 800);
            }, 1500);
          } else {
            new Notice(t('notice_launch_failed', { error: result.error || t('error_unknown') }));
          }
        };

        const retryBtn = btnGroup.createEl('button', { text: t('status_btn_retry') });
        retryBtn.onclick = () => {
          new Notice(t('notice_reconnecting'));
          this.plugin.sensor.reconnectBridge();
          setTimeout(() => this.display(), 600);
        };
      } else {
        pathInfo.setText(t('status_no_path_found'));
        const btnGroup = actions.createDiv({ cls: 'cuesor-btn-group' });
        const retryBtn = btnGroup.createEl('button', { text: t('status_btn_retry') });
        retryBtn.onclick = () => {
          new Notice(t('notice_reconnecting'));
          this.plugin.sensor.reconnectBridge();
          setTimeout(() => this.display(), 600);
        };
      }
    }
  }

  private renderCoreSettings(containerEl: HTMLElement): void {
    new Setting(containerEl)
      .setName(t('setting_indicator_mode_name'))
      .setDesc(t('setting_indicator_mode_desc'))
      .addDropdown((dropdown) => {
        dropdown
          .addOption('both', t('setting_indicator_mode_both'))
          .addOption('color', t('setting_indicator_mode_color'))
          .addOption('badge', t('setting_indicator_mode_badge'))
          .addOption('none', t('setting_indicator_mode_none'))
          .setValue(this.plugin.settings.indicatorMode)
          .onChange(async (value) => {
            this.plugin.settings.indicatorMode = value as IndicatorMode;
            await this.plugin.saveSettings();
            this.display();
          });
      });
  }

  private renderColorSettings(containerEl: HTMLElement): void {
    containerEl.createEl('h3', { text: t('settings_group_color') });

    const effective = this.plugin.store.getEffectiveColors();

    new Setting(containerEl)
      .setName(t('setting_sync_cuesor_name'))
      .setDesc(t('setting_sync_cuesor_desc'))
      .addToggle((toggle) => {
        toggle
          .setValue(this.plugin.settings.colors.syncWithCuesor)
          .onChange(async (value) => {
            this.plugin.settings.colors.syncWithCuesor = value;
            await this.plugin.saveSettings();
            this.display();
          });
      });

    if (this.plugin.settings.colors.syncWithCuesor) {
      const swatchContainer = containerEl.createDiv({ cls: 'cuesor-color-swatches' });

      const zhItem = swatchContainer.createDiv({ cls: 'cuesor-swatch-item' });
      const zhCircle = zhItem.createSpan({ cls: 'cuesor-color-circle' });
      zhCircle.style.backgroundColor = effective.zh;
      zhItem.createSpan({ text: t('setting_swatch_zh', { color: effective.zh }) });

      const enItem = swatchContainer.createDiv({ cls: 'cuesor-swatch-item' });
      const enCircle = enItem.createSpan({ cls: 'cuesor-color-circle' });
      enCircle.style.backgroundColor = effective.en;
      enItem.createSpan({ text: t('setting_swatch_en', { color: effective.en }) });

      const refreshBtn = swatchContainer.createEl('button', {
        text: t('setting_btn_refresh_colors'),
      });
      refreshBtn.onclick = () => {
        const colors = readCuesorSettingsColors();
        if (colors) {
          this.plugin.store.updateCuesorColors(colors);
          new Notice(t('notice_colors_updated', { zh: colors.zh, en: colors.en }));
        } else {
          new Notice(t('notice_reading_config_failed'));
        }
        this.display();
      };
    } else {
      new Setting(containerEl)
        .setName(t('setting_custom_zh_name'))
        .setDesc(t('setting_custom_zh_desc'))
        .addColorPicker((color) => {
          color
            .setValue(this.plugin.settings.colors.zh || effective.zh)
            .onChange(async (value) => {
              this.plugin.settings.colors.zh = value;
              await this.plugin.saveSettings();
            });
        });

      new Setting(containerEl)
        .setName(t('setting_custom_en_name'))
        .setDesc(t('setting_custom_en_desc'))
        .addColorPicker((color) => {
          color
            .setValue(this.plugin.settings.colors.en || effective.en)
            .onChange(async (value) => {
              this.plugin.settings.colors.en = value;
              await this.plugin.saveSettings();
            });
        });
    }
  }

  private renderBadgeSettings(containerEl: HTMLElement): void {
    const mode = this.plugin.settings.indicatorMode;
    if (mode !== 'both' && mode !== 'badge') {
      return;
    }

    containerEl.createEl('h3', { text: t('settings_group_badge') });

    new Setting(containerEl)
      .setName(t('setting_badge_zh_name'))
      .setDesc(t('setting_badge_zh_desc'))
      .addText((text) => {
        text
          .setPlaceholder(t('setting_badge_zh_placeholder'))
          .setValue(this.plugin.settings.badge.textZh)
          .onChange(async (value) => {
            this.plugin.settings.badge.textZh = value.trim() || '中';
            await this.plugin.saveSettings();
          });
      });

    new Setting(containerEl)
      .setName(t('setting_badge_en_name'))
      .setDesc(t('setting_badge_en_desc'))
      .addText((text) => {
        text
          .setPlaceholder(t('setting_badge_en_placeholder'))
          .setValue(this.plugin.settings.badge.textEn)
          .onChange(async (value) => {
            this.plugin.settings.badge.textEn = value.trim() || 'EN';
            await this.plugin.saveSettings();
          });
      });
  }

  private renderBridgeSettings(containerEl: HTMLElement): void {
    containerEl.createEl('h3', { text: t('settings_group_bridge') });

    new Setting(containerEl)
      .setName(t('setting_cuesor_path_name'))
      .setDesc(t('setting_cuesor_path_desc'))
      .addText((text) => {
        text
          .setPlaceholder(t('setting_cuesor_path_placeholder'))
          .setValue(this.plugin.settings.bridge.cuesorPath || '')
          .onChange(async (value) => {
            this.plugin.settings.bridge.cuesorPath = value.trim();
            await this.plugin.saveSettings();

            if (this.pathRefreshTimer !== null) {
              window.clearTimeout(this.pathRefreshTimer);
            }
            this.pathRefreshTimer = window.setTimeout(() => {
              this.pathRefreshTimer = null;
              void this.plugin.sensor.refreshCuesorInfo();
            }, 500);
          });
      });
  }
}
