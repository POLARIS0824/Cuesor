import { findCuesorPath, readCuesorSettingsColors, CuesorColors } from './cuesor-locator';
import { StateStore } from './state-store';
import { ImeMode } from './types';

export class ImeSensor {
  private readonly store: StateStore;
  private ws: WebSocket | null = null;
  private reconnectTimer: number | null = null;
  private isDestroyed = false;

  private readonly boundOnCompositionStart = this.onCompositionStart.bind(this);
  private readonly boundOnCompositionEnd = this.onCompositionEnd.bind(this);
  private readonly boundOnBeforeInput = this.onBeforeInput.bind(this);

  constructor(store: StateStore) {
    this.store = store;
  }

  public async initialize(): Promise<void> {
    this.registerDomListeners();
    await this.refreshCuesorInfo();
    this.connectBridge();
  }

  public async refreshCuesorInfo(): Promise<void> {
    const colors = readCuesorSettingsColors();
    if (colors) {
      this.store.updateCuesorColors(colors);
    }
    const { bridge } = this.store.getSnapshot().settings;
    const resolvedPath = await findCuesorPath(bridge.cuesorPath);
    this.store.updateCuesorPath(resolvedPath);
  }

  public toggleManualMode(): void {
    const current = this.store.getSnapshot().mode;
    const next: ImeMode = current === 'zh' ? 'en' : 'zh';
    this.store.updateIme(next, 'manual-toggle');
  }

  public reconnectBridge(): void {
    this.disconnectBridge();
    this.refreshCuesorInfo();
    this.connectBridge();
  }

  private connectBridge(): void {
    if (this.isDestroyed) {
      return;
    }
    const { bridge } = this.store.getSnapshot().settings;
    if (!bridge.enabled) {
      this.store.updateBridgeStatus(false);
      return;
    }

    try {
      const url = `ws://${bridge.host}:${bridge.port}/cuesor`;
      const socket = new WebSocket(url);
      this.ws = socket;

      socket.onopen = () => {
        if (this.ws === socket) {
          this.store.updateBridgeStatus(true);
          this.refreshCuesorInfo();
        }
      };

      socket.onmessage = (event: MessageEvent) => {
        try {
          const data = JSON.parse(String(event.data)) as {
            mode?: string;
            type?: string;
            colors?: CuesorColors;
          };
          if (data && data.colors && data.colors.zh && data.colors.en) {
            this.store.updateCuesorColors(data.colors);
          }
          if (data && (data.mode === 'zh' || data.mode === 'en' || data.mode === 'unknown')) {
            this.store.updateIme(data.mode as ImeMode, 'cuesor-bridge');
          }
        } catch {
          // ignore malformed messages
        }
      };

      socket.onerror = () => {
        if (this.ws === socket) {
          this.store.updateBridgeStatus(false);
        }
      };

      socket.onclose = () => {
        if (this.ws === socket) {
          this.ws = null;
          this.store.updateBridgeStatus(false);
        }
        if (!this.isDestroyed && bridge.autoReconnect && bridge.enabled) {
          if (this.reconnectTimer === null) {
            this.reconnectTimer = window.setTimeout ? window.setTimeout(() => {
              this.reconnectTimer = null;
              this.connectBridge();
            }, 5000) as unknown as number : setTimeout(() => {
              this.reconnectTimer = null;
              this.connectBridge();
            }, 5000) as unknown as number;
          }
        }
      };
    } catch {
      this.store.updateBridgeStatus(false);
    }
  }

  private disconnectBridge(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      const socket = this.ws;
      this.ws = null;
      socket.onopen = null;
      socket.onmessage = null;
      socket.onerror = null;
      socket.onclose = null;
      try {
        socket.close();
      } catch {
        // ignore
      }
    }
    this.store.updateBridgeStatus(false);
  }

  private registerDomListeners(): void {
    window.addEventListener('compositionstart', this.boundOnCompositionStart, true);
    window.addEventListener('compositionend', this.boundOnCompositionEnd, true);
    window.addEventListener('beforeinput', this.boundOnBeforeInput, true);
  }

  private unregisterDomListeners(): void {
    window.removeEventListener('compositionstart', this.boundOnCompositionStart, true);
    window.removeEventListener('compositionend', this.boundOnCompositionEnd, true);
    window.removeEventListener('beforeinput', this.boundOnBeforeInput, true);
  }

  private onCompositionStart(): void {
    this.store.updateIme('zh', 'dom-composition');
  }

  private onCompositionEnd(event: CompositionEvent): void {
    if (event.data && /[\u4e00-\u9fa5]/.test(event.data)) {
      this.store.updateIme('zh', 'dom-composition');
    }
  }

  private onBeforeInput(event: InputEvent): void {
    if (event.inputType === 'insertCompositionText') {
      this.store.updateIme('zh', 'dom-composition');
    }
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.disconnectBridge();
    this.unregisterDomListeners();
  }
}
