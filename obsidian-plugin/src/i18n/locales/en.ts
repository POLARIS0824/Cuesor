/**
 English translations for Cuesor Obsidian Plugin
 */
export const en = {
  // Commands
  command_toggle_indicator_mode: 'Toggle indicator mode (Color / Badge / Both / Disabled)',
  command_launch_cuesor: 'Launch Cuesor desktop app',
  command_reconnect_bridge: 'Reconnect to Cuesor desktop',
  command_toggle_simulated_ime: 'Simulate IME mode toggle (Chinese / English)',

  // Notices
  notice_indicator_mode_changed: 'Cuesor indicator mode changed to: {mode}',
  notice_already_connected: 'Cuesor desktop is already running and connected.',
  notice_launching_cuesor: 'Launching Cuesor desktop app...',
  notice_launch_failed: 'Launch failed: {error}',
  notice_reconnected_success: 'Successfully connected to Cuesor desktop.',
  notice_disconnected_header: 'Not connected to Cuesor desktop. ',
  notice_click_to_launch: 'Click to launch Cuesor',
  notice_disconnected_guide: 'Please configure the executable path in settings or start Cuesor manually.',
  notice_reading_config_failed: 'Failed to read Cuesor settings file. Please ensure Cuesor is installed and running.',
  notice_colors_updated: 'Updated Cuesor colors: Chinese {zh}, English {en}',
  notice_reconnecting: 'Reconnecting to Cuesor...',

  // Settings tab headers
  settings_title: 'Cuesor - Cursor & IME Status Indicator',
  settings_group_core: 'Indicator Mode',
  settings_group_color: 'Cursor Color Settings',
  settings_group_badge: 'Floating Badge Settings',
  settings_group_bridge: 'Desktop Bridge Settings',

  // Status card
  status_title_connected: 'Cuesor Desktop: Connected',
  status_title_disconnected: 'Cuesor Desktop: Disconnected',
  status_desc_connected: 'Connected via local port {port}. Synchronizing Win32 IME status and cursor colors in real time.',
  status_desc_disconnected: 'Cuesor is not detected or cannot connect to port {port}. Run Cuesor to enable system-level precise IME status sync.',
  status_detected_path: 'Detected executable path: ',
  status_btn_launch: 'Launch Cuesor',
  status_btn_retry: 'Retry Connection',
  status_no_path_found: 'Cuesor installation path was not detected automatically. You can specify it manually in advanced settings below, or start Cuesor manually.',

  // Core settings
  setting_indicator_mode_name: 'Indicator display mode',
  setting_indicator_mode_desc: 'Choose to indicate Chinese/English input mode with cursor color, floating text badge, or both.',
  setting_indicator_mode_both: 'Both cursor color and badge',
  setting_indicator_mode_color: 'Cursor color only',
  setting_indicator_mode_badge: 'Floating badge only',
  setting_indicator_mode_none: 'Disable indicator',

  // Color settings
  setting_sync_cuesor_name: 'Sync with Cuesor colors',
  setting_sync_cuesor_desc: 'Enabled by default. Automatically syncs with the active color scheme in the Cuesor desktop app.',
  setting_swatch_zh: 'Chinese mode: {color}',
  setting_swatch_en: 'English mode: {color}',
  setting_btn_refresh_colors: 'Reload colors from Cuesor',
  setting_custom_zh_name: 'Custom Chinese mode color',
  setting_custom_zh_desc: 'Caret and badge color when in Chinese IME mode.',
  setting_custom_en_name: 'Custom English mode color',
  setting_custom_en_desc: 'Caret and badge color when in English IME mode.',

  // Badge settings
  setting_badge_zh_name: 'Chinese badge text',
  setting_badge_zh_desc: 'Text displayed on the floating badge in Chinese mode.',
  setting_badge_zh_placeholder: '中',
  setting_badge_en_name: 'English badge text',
  setting_badge_en_desc: 'Text displayed on the floating badge in English mode.',
  setting_badge_en_placeholder: 'EN',

  // Bridge settings
  setting_bridge_enable_name: 'Connect to Cuesor desktop engine',
  setting_bridge_enable_desc: 'Receive Windows IME status in real time via local WebSocket bridge.',
  setting_cuesor_path_name: 'Custom Cuesor executable path',
  setting_cuesor_path_desc: 'If Cuesor.exe was not detected automatically, enter its full absolute path here.',
  setting_cuesor_path_placeholder: 'e.g. D:\\Code\\Cuesor\\target\\debug\\Cuesor.exe',
  setting_bridge_port_name: 'Bridge service port',
  setting_bridge_port_desc: 'Local WebSocket port used by Cuesor to broadcast state (default: 23333).',

  // System errors
  error_windows_only: 'Cuesor only supports Windows.',
  error_process_spawn_failed: 'Unable to invoke system process component.',
  error_executable_not_found: 'Executable does not exist: {path}',
  error_unknown: 'Unknown error',
};
