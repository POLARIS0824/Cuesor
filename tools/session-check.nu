# Run with nu -n -e 'source .../tools/session-check.nu' inside Windows Terminal.
const root = path self ..
let tool = ($root | path join target release cuesor-engine.exe)
let artifacts = ($root | path join artifacts)
mkdir $artifacts
let session = $env.WT_SESSION
let log = ($artifacts | path join $'worker-($session).jsonl')
^$tool start --shell-pid $nu.pid --log $log
mut ready = false
for attempt in 1..40 {
    sleep 25ms
    let state = (^$tool status | from json)
    if $state.running { $ready = true; break }
}
# Source the actual integration too; this must not create another worker.
source ../integrations/nushell.nu
let duplicate_start = (timeit { ^$tool start --shell-pid $nu.pid })
{
    shell_pid: $nu.pid
    session: $session
    running: $ready
    duplicate_start_ms: ($duplicate_start / 1ms)
    log: $log
} | to json | save --force ($artifacts | path join $'session-($session).json')
print $'Cuesor running: ($ready). Chinese=orange, English=blue.'
print 'This is a normal editable Nu prompt. Try IME switching, typing, completion and Ctrl+C.'
print 'Run ./target/release/cuesor-engine.exe stop to stop and restore the theme color.'
