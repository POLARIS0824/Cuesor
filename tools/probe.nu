# Isolated, temporary Windows Terminal validation. Does not load/edit user config.
const root = path self ..
let tool = ($root | path join target release terminal-ime.exe)
let artifacts = ($root | path join artifacts)
mkdir $artifacts
print 'Terminal IME native-color probe (temporary session)'
print 'This test continuously changes the cursor: Chinese=orange, English=blue.'
print 'Only window/IME metadata is recorded; typed text is not recorded.'
input 'Press Enter when ready; then switch Chinese/English several times for 20 seconds: '
^$tool probe --follow --watch 20 --report ($artifacts | path join follow-probe.jsonl)
let worked = (input 'Did the cursor visibly switch between orange and blue? Type y or n: ')
{color_follow_ok: (($worked | str trim | str downcase) == 'y')} | to json | save --force ($artifacts | path join follow-verdict.json)
print $'Probe complete. Reports: ($artifacts)'
