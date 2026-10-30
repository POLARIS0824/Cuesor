import { findCuesorPath, readCuesorSettingsColors, CuesorColors } from './cuesor-locator';
import { StateStore } from './state-store';
import { BRIDGE_HOST, BRIDGE_PORT, ImeMode } from './types';

const RECONNECT_DELAY_MS = 5000;

export class ImeSensor {
  private readonly store: StateStore;
  private ws: WebSocket | null = null;
  private reconnectTimer: number | null = null;
  private isDestroyed = false;

  constructor(store: StateStore) {
    this.store = store;
  }

  public initialize(): void {
    this.connectBridge();
    void this.refreshCuesorInfo();
  }

  public async refreshCuesorInfo(): Promise<void> {
    const colors = readCuesorSettingsColors();
    if (colors) {
      this.store.updateCuesorColors(colors);
    }
    const { cuesorPath } = this.store.getSnapshot().settings.bridge;
    const resolvedPath = await findCuesorPath(cuesorPath);
    this.store.updateCuesorPath(resolvedPath);
  }

  public reconnectBridge(): void {
    if (this.isDestroyed) {
      return;
    }
    this.disconnectBridge();
    void this.refreshCuesorInfo();
    this.connectBridge();
  }

  private connectBridge(): void {
    if (this.isDestroyed || this.ws) {
      return;
    }

    try {
      const socket = new WebSocket(`ws://${BRIDGE_HOST}:${BRIDGE_PORT}/cuesor`);
      this.ws = socket;

      socket.onopen = () => {
        if (this.ws === socket) {
          this.store.updateBridgeStatus(true);
        }
      };

      socket.onmessage = (event: MessageEvent) => {
        try {
          const data = JSON.parse(String(event.data)) as {
            mode?: string;
            colors?: CuesorColors;
          };
          if (data.colors?.zh && data.colors.en) {
            this.store.updateCuesorColors(data.colors);
          }
          if (data.mode === 'zh' || data.mode === 'en' || data.mode === 'unknown') {
            this.store.updateIme(data.mode as ImeMode);
          }
        } catch {
          // ignore malformed messages
        }
      };

      socket.onerror = () => {
        if (this.ws === socket) {
          this.markDisconnected();
        }
      };

      socket.onclose = () => {
        if (this.ws === socket) {
          this.ws = null;
          this.markDisconnected();
        }
        this.scheduleReconnect();
      };
    } catch {
      this.markDisconnected();
      this.scheduleReconnect();
    }
  }

  private markDisconnected(): void {
    this.store.updateBridgeStatus(false);
    this.store.updateIme('unknown');
  }

  private scheduleReconnect(): void {
    if (this.isDestroyed || this.reconnectTimer !== null) {
      return;
    }
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connectBridge();
    }, RECONNECT_DELAY_MS) as unknown as number;
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
    this.markDisconnected();
  }

  public destroy(): void {
    this.isDestroyed = true;
    this.disconnectBridge();
  }
}
