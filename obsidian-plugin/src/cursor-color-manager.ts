import { StateSnapshot } from './state-store';

export class CursorColorManager {
  private styleEl: HTMLStyleElement | null = null;

  public initialize(): void {
    if (!this.styleEl) {
      this.styleEl = document.createElement('style');
      this.styleEl.id = 'cuesor-cursor-styles';
      document.head.appendChild(this.styleEl);
    }
  }

  public render(snapshot: StateSnapshot): void {
    if (!this.styleEl) {
      this.initialize();
    }
    if (!this.styleEl) {
      return;
    }

    const { mode, settings, cuesorColors } = snapshot;
    const { indicatorMode, colors } = settings;

    const isColorEnabled = indicatorMode === 'color' || indicatorMode === 'both';
    const isBadgeEnabled = indicatorMode === 'badge' || indicatorMode === 'both';

    if (!isColorEnabled && !isBadgeEnabled) {
      this.styleEl.textContent = '';
      return;
    }

    // Resolve effective colors: default to syncing with Cuesor's active colors
    const effectiveZh = colors.syncWithCuesor ? cuesorColors.zh : (colors.zh || cuesorColors.zh);
    const effectiveEn = colors.syncWithCuesor ? cuesorColors.en : (colors.en || cuesorColors.en);

    let activeColor: string;
    switch (mode) {
      case 'zh':
        activeColor = effectiveZh;
        break;
      case 'en':
        activeColor = effectiveEn;
        break;
      case 'unknown':
      default:
        activeColor = 'var(--text-accent, #705dcf)';
        break;
    }

    const rules: string[] = [
      '/* Cuesor cursor & badge styling */',
    ];

    if (isBadgeEnabled) {
      rules.push(
        ':root {',
        `  --cuesor-badge-bg-zh: ${effectiveZh};`,
        `  --cuesor-badge-bg-en: ${effectiveEn};`,
        '}'
      );
    }

    if (isColorEnabled) {
      rules.push(
        '.markdown-source-view.mod-cm6 .cm-content,',
        '.markdown-source-view.mod-cm6 .cm-line {',
        `  caret-color: ${activeColor} !important;`,
        '}',
        '.cm-editor .cm-cursor {',
        `  border-left-color: ${activeColor} !important;`,
        '  border-left-width: 2px !important;',
        '}'
      );
    }

    const css = rules.join('\n');
    if (this.styleEl.textContent !== css) {
      this.styleEl.textContent = css;
    }
  }

  public destroy(): void {
    if (this.styleEl && this.styleEl.parentNode) {
      this.styleEl.parentNode.removeChild(this.styleEl);
    }
    this.styleEl = null;
  }
}
