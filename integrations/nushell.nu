# Add one top-level `source /absolute/path/to/Terminal_IME/integrations/nushell.nu`
# to config.nu after validating the release build. No hooks or background Nu jobs.
const terminal_ime_executable = path self ../target/release/terminal-ime.exe
if $nu.is-interactive and (($env.WT_SESSION? | default '') != '') {
    if ($terminal_ime_executable | path exists) {
        try { ^$terminal_ime_executable start --shell-pid $nu.pid | complete | ignore } catch { }
    }
}
