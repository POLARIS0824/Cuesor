import { App, MarkdownView } from 'obsidian';
import { StateSnapshot } from './state-store';

export class CursorBadgeManager {
  private badgeEl: HTMLDivElement | null = null;
  private readonly app: App;
  private latestSnapshot: StateSnapshot | null = null;
  private frame: number | null = null;
  private destroyed = false;
  private lastLabel: string | null = null;
  private lastMode: string | null = null;
  private badgeWidth = 0;
  private badgeHeight = 0;
  private lastLeft = Number.NaN;
  private lastTop = Number.NaN;

  constructor(app: App) {
    this.app = app;
  }

  public initialize(): void {
    if (!this.badgeEl) {
      this.badgeEl = document.createElement('div');
      this.badgeEl.id = 'cuesor-cursor-badge';
      this.badgeEl.setAttribute('aria-hidden', 'true');
      document.body.appendChild(this.badgeEl);
    }
  }

  public render(snapshot: StateSnapshot): void {
    if (this.destroyed) {
      return;
    }
    this.latestSnapshot = snapshot;
    this.updatePosition();
  }

  public scheduleUpdate(): void {
    if (this.destroyed || this.frame !== null) {
      return;
    }
    this.frame = window.requestAnimationFrame(() => {
      this.frame = null;
      this.updatePosition();
    });
  }

  public updatePosition(): void {
    if (this.destroyed) {
      return;
    }
    if (!this.badgeEl) {
      this.initialize();
    }
    if (!this.badgeEl || !this.latestSnapshot) {
      return;
    }

    const { mode, settings } = this.latestSnapshot;
    const { indicatorMode, badge } = settings;

    const isBadgeEnabled = indicatorMode === 'badge' || indicatorMode === 'both';
    if (!isBadgeEnabled) {
      this.hide();
      return;
    }

    if (typeof document.hasFocus === 'function' && !document.hasFocus()) {
      this.hide();
      return;
    }

    if (!this.app.workspace || typeof this.app.workspace.getActiveViewOfType !== 'function') {
      this.hide();
      return;
    }

    const activeView = this.app.workspace.getActiveViewOfType(MarkdownView);
    if (!activeView || activeView.getMode() !== 'source') {
      this.hide();
      return;
    }

    const editor = activeView.editor as unknown as {
      getCursor: () => { line: number; ch: number };
      posToOffset: (pos: { line: number; ch: number }) => number;
      cm?: {
        hasFocus: boolean;
        coordsAtPos: (offset: number) => { left: number; top: number; right: number; bottom: number } | null;
        scrollDOM: HTMLElement;
      };
    };

    const contentEl = activeView.containerEl.querySelector('.cm-content') as unknown as {
      cmView?: {
        view?: {
          hasFocus: boolean;
          coordsAtPos: (offset: number) => { left: number; top: number; right: number; bottom: number } | null;
          scrollDOM: HTMLElement;
        };
      };
    };

    const cm = editor.cm || contentEl?.cmView?.view;
    if (!cm || typeof cm.coordsAtPos !== 'function') {
      this.hide();
      return;
    }

    if (!cm.hasFocus) {
      this.hide();
      return;
    }

    try {
      const cursor = editor.getCursor();
      const offset = editor.posToOffset(cursor);
      const caret = cm.coordsAtPos(offset);
      if (!caret) {
        this.hide();
        return;
      }

      const viewport = cm.scrollDOM.getBoundingClientRect();
      const containerBounds = activeView.containerEl.getBoundingClientRect();

      const clip = {
        left: Math.max(viewport.left, containerBounds.left, 0),
        top: Math.max(viewport.top, containerBounds.top, 0),
        right: Math.min(viewport.right, containerBounds.right, window.innerWidth),
        bottom: Math.min(viewport.bottom, containerBounds.bottom, window.innerHeight),
      };

      if (
        caret.left < clip.left ||
        caret.left > clip.right ||
        caret.bottom < clip.top ||
        caret.top > clip.bottom
      ) {
        this.hide();
        return;
      }

      let labelText: string;
      switch (mode) {
        case 'zh':
          labelText = badge.textZh;
          break;
        case 'en':
          labelText = badge.textEn;
          break;
        case 'unknown':
        default:
          labelText = badge.textUnknown;
          break;
      }

      if (this.badgeEl.style.display !== 'block') {
        this.badgeEl.style.display = 'block';
      }

      if (labelText !== this.lastLabel) {
        this.badgeEl.textContent = labelText;
        this.lastLabel = labelText;
        this.badgeWidth = 0;
        this.badgeHeight = 0;
      }
      if (mode !== this.lastMode) {
        this.badgeEl.setAttribute('data-mode', mode);
        this.lastMode = mode;
      }

      const badgeWidth = this.badgeWidth || (this.badgeWidth = this.badgeEl.offsetWidth || 30);
      const badgeHeight = this.badgeHeight || (this.badgeHeight = this.badgeEl.offsetHeight || 18);

      let x = caret.left + badge.offsetX;
      x = Math.max(clip.left, Math.min(x, clip.right - badgeWidth));

      let y = caret.bottom + badge.offsetY;
      if (y + badgeHeight > clip.bottom && caret.top - badgeHeight - badge.offsetY >= clip.top) {
        y = caret.top - badgeHeight - badge.offsetY;
      } else {
        y = Math.max(clip.top, Math.min(y, clip.bottom - badgeHeight));
      }

      const left = Math.round(x);
      const top = Math.round(y);
      if (left !== this.lastLeft || top !== this.lastTop) {
        this.badgeEl.style.left = `${left}px`;
        this.badgeEl.style.top = `${top}px`;
        this.lastLeft = left;
        this.lastTop = top;
      }
    } catch {
      this.hide();
    }
  }

  public hide(): void {
    if (this.badgeEl && this.badgeEl.style.display !== 'none') {
      this.badgeEl.style.display = 'none';
    }
  }

  public destroy(): void {
    this.destroyed = true;
    if (this.frame !== null) {
      window.cancelAnimationFrame(this.frame);
      this.frame = null;
    }
    if (this.badgeEl && this.badgeEl.parentNode) {
      this.badgeEl.parentNode.removeChild(this.badgeEl);
    }
    this.badgeEl = null;
  }
}
