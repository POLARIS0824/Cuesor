import { FALLBACK_CUESOR_COLORS, CuesorColors } from './cuesor-locator';
import { CuesorPluginSettings, DEFAULT_SETTINGS, ImeMode, ImeSource } from './types';

export interface StateSnapshot {
  readonly mode: ImeMode;
  readonly source: ImeSource;
  readonly bridgeConnected: boolean;
  readonly cuesorColors: CuesorColors;
  readonly cuesorPath: string | null;
  readonly settings: CuesorPluginSettings;
}

export type StateListener = (snapshot: StateSnapshot) => void;

export class StateStore {
  private mode: ImeMode = 'en';
  private source: ImeSource = 'dom-composition';
  private bridgeConnected = false;
  private cuesorColors: CuesorColors = { ...FALLBACK_CUESOR_COLORS };
  private cuesorPath: string | null = null;
  private settings: CuesorPluginSettings = { ...DEFAULT_SETTINGS };
  private readonly listeners = new Set<StateListener>();

  constructor(initialSettings?: CuesorPluginSettings) {
    if (initialSettings) {
      this.settings = initialSettings;
    }
  }

  public getSnapshot(): StateSnapshot {
    return {
      mode: this.mode,
      source: this.source,
      bridgeConnected: this.bridgeConnected,
      cuesorColors: this.cuesorColors,
      cuesorPath: this.cuesorPath,
      settings: this.settings,
    };
  }

  public updateIme(mode: ImeMode, source: ImeSource): void {
    if (this.mode === mode && this.source === source) {
      return;
    }
    this.mode = mode;
    this.source = source;
    this.notify();
  }

  public updateBridgeStatus(connected: boolean): void {
    if (this.bridgeConnected === connected) {
      return;
    }
    this.bridgeConnected = connected;
    this.notify();
  }

  public updateCuesorColors(colors: CuesorColors): void {
    if (this.cuesorColors.zh === colors.zh && this.cuesorColors.en === colors.en) {
      return;
    }
    this.cuesorColors = { ...colors };
    this.notify();
  }

  public updateCuesorPath(path: string | null): void {
    if (this.cuesorPath === path) {
      return;
    }
    this.cuesorPath = path;
    this.notify();
  }

  public updateSettings(settings: CuesorPluginSettings): void {
    this.settings = settings;
    this.notify();
  }

  public getEffectiveColors(): CuesorColors {
    if (this.settings.colors.syncWithCuesor) {
      return this.cuesorColors;
    }
    return {
      zh: this.settings.colors.zh || FALLBACK_CUESOR_COLORS.zh,
      en: this.settings.colors.en || FALLBACK_CUESOR_COLORS.en,
    };
  }

  public subscribe(listener: StateListener): () => void {
    this.listeners.add(listener);
    listener(this.getSnapshot());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snapshot = this.getSnapshot();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (error) {
        console.error('Cuesor StateStore listener error:', error);
      }
    }
  }
}
