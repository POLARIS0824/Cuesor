/**
 * Cuesor Obsidian Plugin Domain Types
 */

export type ImeMode = 'zh' | 'en' | 'unknown';

export type IndicatorMode = 'color' | 'badge' | 'both' | 'none';

export type ImeSource = 'cuesor-bridge' | 'dom-composition' | 'manual-toggle';

export interface CaretCoordinates {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
}

export interface ColorSchemeConfig {
  syncWithCuesor: boolean;
  zh: string;
  en: string;
  // Legacy fields for backward compatibility migration
  zhLight?: string;
  zhDark?: string;
  enLight?: string;
  enDark?: string;
}

export interface BadgeConfig {
  textZh: string;
  textEn: string;
  textUnknown: string;
  offsetY: number;
  offsetX: number;
  showSourceTag: boolean;
}

export interface BridgeConfig {
  enabled: boolean;
  port: number;
  host: string;
  autoReconnect: boolean;
  cuesorPath: string;
}

export interface CuesorPluginSettings {
  indicatorMode: IndicatorMode;
  colors: ColorSchemeConfig;
  badge: BadgeConfig;
  bridge: BridgeConfig;
}

export const DEFAULT_SETTINGS: CuesorPluginSettings = {
  indicatorMode: 'both',
  colors: {
    syncWithCuesor: true,
    zh: '#F76B15',
    en: '#0090FF',
  },
  badge: {
    textZh: '中',
    textEn: 'EN',
    textUnknown: '?',
    offsetX: 8,
    offsetY: 3,
    showSourceTag: false,
  },
  bridge: {
    enabled: true,
    port: 23333,
    host: '127.0.0.1',
    autoReconnect: true,
    cuesorPath: '',
  },
};
