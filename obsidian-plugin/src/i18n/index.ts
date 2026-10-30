import { en } from './locales/en';
import { zhCn } from './locales/zh-cn';

export type TranslationKey = keyof typeof en;

const locales: Record<string, typeof en> = {
  en,
  zh: zhCn,
  'zh-cn': zhCn,
  'zh-tw': zhCn,
  'zh-hk': zhCn,
};

/**
 * Detect current active language from Obsidian environment.
 */
export function getLanguage(): string {
  if (typeof window !== 'undefined') {
    const custom = window.localStorage.getItem('language');
    if (custom) {
      return custom.toLowerCase();
    }
    const momentLocale = (window as any).moment?.locale?.();
    if (momentLocale) {
      return String(momentLocale).toLowerCase();
    }
  }
  return 'en';
}

/**
 * Translate a key into the active language, with variable interpolation and fallback to English.
 */
export function t(key: TranslationKey, params?: Record<string, string | number>): string {
  const lang = getLanguage();
  const locale = locales[lang] || (lang.startsWith('zh') ? locales['zh-cn'] : locales.en);
  let template = locale[key] || en[key] || key;

  if (params) {
    for (const [k, v] of Object.entries(params)) {
      template = template.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
    }
  }

  return template;
}
