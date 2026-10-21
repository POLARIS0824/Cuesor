# Run after normal user config has loaded, with --no-history to avoid recording tests.
const root = path self ..
let tool = ($root | path join target release terminal-ime.exe)
let artifacts = ($root | path join artifacts)
mkdir $artifacts
let before_keys = ($env.config.keybindings | get name)
let before_prompt = ($env.config.hooks.pre_prompt | length)
let before_execution = ($env.config.hooks.pre_execution | length)
source ../integrations/nushell.nu
mut running = false
for attempt in 1..40 {
    sleep 25ms
    if (^$tool status | from json | get running) { $running = true; break }
}
{
    running: $running
    shell_pid: $nu.pid
    session: $env.WT_SESSION
    keybindings_unchanged: ($before_keys == ($env.config.keybindings | get name))
    hooks_unchanged: (($before_prompt == ($env.config.hooks.pre_prompt | length)) and ($before_execution == ($env.config.hooks.pre_execution | length)))
    atuin_version: (atuin --version)
} | to json | save --force ($artifacts | path join atuin-check.json)
print $'Terminal IME + actual Nu/Atuin config. Worker running: ($running)'
print 'Try Ctrl+R and Up, cancel/accept a history result, then switch IME and continue editing.'
print 'Do not execute a history result unless you intend to. No screen/history contents are captured.'
print 'This test session has Nu history recording disabled.'
