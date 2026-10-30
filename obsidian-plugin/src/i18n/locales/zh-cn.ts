import type { en } from './en';

/**
 * 简体中文翻译 (Cuesor Obsidian Plugin)
 */
export const zhCn: typeof en = {
  // Commands
  command_toggle_indicator_mode: '切换指示模式 (Color / Badge / Both / Disabled)',
  command_launch_cuesor: '启动 Cuesor 桌面端',
  command_reconnect_bridge: '重新连接 Cuesor 桌面端',

  // Notices
  notice_indicator_mode_changed: 'Cuesor 指示模式已切换为: {mode}',
  notice_already_connected: 'Cuesor 桌面端已在运行并处于连接状态。',
  notice_launching_cuesor: '正在启动 Cuesor 桌面端...',
  notice_launch_failed: '启动失败: {error}',
  notice_reconnected_success: '已成功连接到 Cuesor 桌面端。',
  notice_disconnected_header: '未连接到 Cuesor 桌面端。 ',
  notice_click_to_launch: '点击启动 Cuesor',
  notice_disconnected_guide: '请在设置中配置程序路径或手动打开 Cuesor。',
  notice_reading_config_failed: '未能读取到 Cuesor 本地配置文件，请确认 Cuesor 已安装运行。',
  notice_colors_updated: '已更新 Cuesor 配色：中文 {zh}，英文 {en}',
  notice_reconnecting: '正在重新尝试连接 Cuesor...',

  // Settings tab headers
  settings_title: 'Cuesor - 光标与输入法状态指示器',
  settings_group_core: '指示器显示模式',
  settings_group_color: '光标配色设置',
  settings_group_badge: '浮动徽标设置',
  settings_group_bridge: '桌面端桥接设置',

  // Status card
  status_title_connected: 'Cuesor 桌面端：已连接',
  status_title_disconnected: 'Cuesor 桌面端：未连接',
  status_desc_connected: '已连接到 Cuesor 桌面端（端口 {port}），正在通过本地桥接读取系统输入法状态。',
  status_desc_disconnected: '未检测到 Cuesor 桌面端，或端口 {port} 不可用。插件依赖 Cuesor 读取输入法状态，请先启动桌面端。',
  status_detected_path: '检测到程序路径：',
  status_btn_launch: '启动 Cuesor',
  status_btn_retry: '重试连接',
  status_no_path_found: '未自动检测到 Cuesor 安装路径。可在下方高级设置中手动指定路径，或手动运行 Cuesor。',

  // Core settings
  setting_indicator_mode_name: '指示器显示模式',
  setting_indicator_mode_desc: '选择通过光标颜色、浮动文字徽标或两者共同指示中英文输入状态。',
  setting_indicator_mode_both: '同时显示光标颜色与徽标',
  setting_indicator_mode_color: '仅显示光标颜色',
  setting_indicator_mode_badge: '仅显示浮动徽标',
  setting_indicator_mode_none: '禁用指示器',

  // Color settings
  setting_sync_cuesor_name: '跟随 Cuesor 配色',
  setting_sync_cuesor_desc: '默认开启。自动与 Cuesor 桌面端当前活动配置方案的中英文颜色保持同步。',
  setting_swatch_zh: '中文模式：{color}',
  setting_swatch_en: '英文模式：{color}',
  setting_btn_refresh_colors: '从 Cuesor 重新读取配色',
  setting_custom_zh_name: '自定义中文模式颜色',
  setting_custom_zh_desc: '在中文输入法状态下光标与指示器徽标使用的颜色。',
  setting_custom_en_name: '自定义英文模式颜色',
  setting_custom_en_desc: '在英文输入法状态下光标与指示器徽标使用的颜色。',

  // Badge settings
  setting_badge_zh_name: '中文状态徽标文本',
  setting_badge_zh_desc: '处于中文输入状态时浮动徽标显示的文字。',
  setting_badge_zh_placeholder: '中',
  setting_badge_en_name: '英文状态徽标文本',
  setting_badge_en_desc: '处于英文输入状态时浮动徽标显示的文字。',
  setting_badge_en_placeholder: 'EN',

  // Bridge settings
  setting_cuesor_path_name: 'Cuesor 自定义可执行文件路径',
  setting_cuesor_path_desc: '若未能自动检测到 Cuesor.exe，可在此输入其完整绝对路径。',
  setting_cuesor_path_placeholder: '例如：D:\\Code\\Cuesor\\target\\debug\\Cuesor.exe',

  // System errors
  error_windows_only: 'Cuesor 仅支持 Windows 系统。',
  error_process_spawn_failed: '无法调用系统进程组件。',
  error_executable_not_found: '可执行文件不存在：{path}',
  error_unknown: '未知错误',
};
