# A noninteractive, short-lived test host. Main is invoked only as a Nu script.
const root = path self ..
def main [report_dir: path] {
    let tool = ($root | path join target release cuesor-engine.exe)
    mkdir $report_dir
    let log = ($report_dir | path join $'worker-($env.WT_SESSION).jsonl')
    ^$tool start --shell-pid $nu.pid --log $log
    mut running = false
    for attempt in 1..80 {
        sleep 25ms
        if (^$tool status | from json | get running) { $running = true; break }
    }
    if not $running { error make {msg: 'load-check worker failed to initialize'} }
    mut metadata = {}
    for attempt in 1..40 {
        let first = (try { open --raw $log | lines | first | from json } catch { {} })
        if 'worker_pid' in $first { $metadata = $first; break }
        sleep 25ms
    }
    $metadata | to json | save --force ($report_dir | path join $'session-($env.WT_SESSION).json')
    sleep 180sec
}
