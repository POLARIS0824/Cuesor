import { t } from './i18n';

/**
 * Utilities to locate, inspect, and launch the Cuesor desktop application.
 */

export interface CuesorColors {
  zh: string;
  en: string;
}

export const FALLBACK_CUESOR_COLORS: CuesorColors = {
  zh: '#F76B15',
  en: '#0090FF',
};

function getModule(name: string): any {
  try {
    // Obsidian desktop exposes node require via window.require or standard require
    if (typeof window !== 'undefined' && (window as any).require) {
      return (window as any).require(name);
    }
    return require(name);
  } catch {
    return null;
  }
}

export function isWindowsPlatform(): boolean {
  if (typeof process === 'undefined') {
    return false;
  }
  return process.platform === 'win32';
}

export function getCuesorDataDir(): string {
  if (typeof process === 'undefined') {
    return '';
  }
  const custom = process.env.CUESOR_DATA_DIR;
  if (custom && custom.trim().length > 0) {
    return custom.trim();
  }
  const localAppData = process.env.LOCALAPPDATA || '';
  const path = getModule('path');
  if (path && localAppData) {
    return path.join(localAppData, 'Cuesor');
  }
  return '';
}

/**
 * Reads active Chinese and English colors from Cuesor's settings.json.
 */
export function readCuesorSettingsColors(): CuesorColors | null {
  const fs = getModule('fs');
  const path = getModule('path');
  if (!fs || !path) {
    return null;
  }

  const dataDir = getCuesorDataDir();
  if (!dataDir) {
    return null;
  }

  const settingsFile = path.join(dataDir, 'settings.json');
  try {
    if (!fs.existsSync(settingsFile)) {
      return null;
    }
    const content = fs.readFileSync(settingsFile, 'utf8');
    const parsed = JSON.parse(content);
    if (!parsed || !Array.isArray(parsed.color_schemes)) {
      return null;
    }
    const activeIndex = typeof parsed.active_scheme === 'number' ? parsed.active_scheme : 0;
    const activeScheme = parsed.color_schemes[activeIndex] || parsed.color_schemes[0];
    if (activeScheme && activeScheme.chinese_color && activeScheme.english_color) {
      return {
        zh: activeScheme.chinese_color,
        en: activeScheme.english_color,
      };
    }
  } catch (error) {
    console.error('Failed to read Cuesor settings.json:', error);
  }
  return null;
}

/**
 * Discovers the absolute path to Cuesor.exe using multiple heuristics:
 * 1. User-specified custom path
 * 2. executable.txt written by Cuesor in %LOCALAPPDATA%\Cuesor\
 * 3. Windows AppCompatFlags Layers registry
 * 4. Windows Run auto-start registry
 * 5. Standard installation folders (Programs, ProgramFiles, Start Menu)
 * 6. Local development output paths
 */
export async function findCuesorPath(customPath?: string): Promise<string | null> {
  const fs = getModule('fs');
  const path = getModule('path');
  const cp = getModule('child_process');

  if (!fs || !path || !isWindowsPlatform()) {
    return null;
  }

  // 1. User-specified path
  if (customPath && customPath.trim().length > 0) {
    const trimmed = customPath.trim();
    if (fs.existsSync(trimmed)) {
      return trimmed;
    }
  }

  const dataDir = getCuesorDataDir();

  // 2. executable.txt recorded by Cuesor on startup
  if (dataDir) {
    const exeTxtPath = path.join(dataDir, 'executable.txt');
    try {
      if (fs.existsSync(exeTxtPath)) {
        const candidate = fs.readFileSync(exeTxtPath, 'utf8').trim();
        if (candidate.length > 0 && fs.existsSync(candidate)) {
          return candidate;
        }
      }
    } catch {
      // ignore
    }
  }

  // 3. AppCompatFlags Layers registry (populated if Run as Administrator was configured)
  if (cp) {
    try {
      const output = await new Promise<string>((resolve) => {
        cp.exec(
          'reg query "HKCU\\Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers"',
          { timeout: 1500, encoding: 'utf8' },
          (err: any, stdout: string) => {
            if (err || !stdout) {
              resolve('');
            } else {
              resolve(stdout);
            }
          }
        );
      });

      for (const line of output.split(/\r?\n/)) {
        if (line.toLowerCase().includes('cuesor.exe')) {
          const matched = line.trim().split(/\s{4,}/)[0];
          if (matched && fs.existsSync(matched)) {
            return matched;
          }
        }
      }
    } catch {
      // ignore
    }

    // 4. Run auto-start registry
    try {
      const runOutput = await new Promise<string>((resolve) => {
        cp.exec(
          'reg query "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run" /v Cuesor',
          { timeout: 1500, encoding: 'utf8' },
          (err: any, stdout: string) => {
            if (err || !stdout) {
              resolve('');
            } else {
              resolve(stdout);
            }
          }
        );
      });

      const parts = runOutput.trim().split(/\s{4,}/);
      if (parts.length >= 3) {
        const rawCmd = parts[2].trim();
        const unquoted = rawCmd.replace(/^"/, '').replace(/"\s*--background.*$/, '').replace(/"$/, '').trim();
        if (unquoted && fs.existsSync(unquoted)) {
          return unquoted;
        }
      }
    } catch {
      // ignore
    }
  }

  // 5. Standard installation folders and shortcuts
  const localAppData = process.env.LOCALAPPDATA || '';
  const appData = process.env.APPDATA || '';
  const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
  const programFilesX86 = process.env['ProgramFiles(x86)'] || 'C:\\Program Files (x86)';

  const commonLocations: string[] = [
    path.join(localAppData, 'Programs', 'Cuesor', 'Cuesor.exe'),
    path.join(localAppData, 'Cuesor', 'Cuesor.exe'),
    path.join(programFiles, 'Cuesor', 'Cuesor.exe'),
    path.join(programFilesX86, 'Cuesor', 'Cuesor.exe'),
    path.join(appData, 'Microsoft', 'Windows', 'Start Menu', 'Programs', 'Cuesor.lnk'),
    // Dev paths relative to known repository locations
    'D:\\Code\\Cuesor\\target\\release\\Cuesor.exe',
    'D:\\Code\\Cuesor\\target\\debug\\Cuesor.exe',
    'D:\\Code\\Cuesor\\artifacts\\Cuesor-portable\\Cuesor.exe',
  ];

  for (const candidate of commonLocations) {
    if (candidate && fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

/**
 * Launches the Cuesor application using Windows Shell (start command),
 * which supports UAC elevation and detached process execution.
 */
export async function launchCuesor(executablePath: string): Promise<{ success: boolean; error?: string }> {
  const fs = getModule('fs');
  const cp = getModule('child_process');

  if (!isWindowsPlatform()) {
    return { success: false, error: t('error_windows_only') };
  }

  if (!cp) {
    return { success: false, error: t('error_process_spawn_failed') };
  }

  if (fs && !fs.existsSync(executablePath)) {
    return { success: false, error: t('error_executable_not_found', { path: executablePath }) };
  }

  return new Promise((resolve) => {
    // Windows "start" automatically opens through ShellExecuteEx, correctly handling elevation prompts and shortcuts
    cp.exec(`start "" "${executablePath}"`, (error: any) => {
      if (error) {
        resolve({ success: false, error: error.message || String(error) });
      } else {
        resolve({ success: true });
      }
    });
  });
}
