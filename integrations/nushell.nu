const cuesor_executable = path self ../target/release/cuesor-engine.exe
if $nu.is-interactive and (($env.WT_SESSION? | default '') != '') {
    if ($cuesor_executable | path exists) {
        try { ^$cuesor_executable start --shell-pid $nu.pid | complete | ignore } catch { }
    }
}
