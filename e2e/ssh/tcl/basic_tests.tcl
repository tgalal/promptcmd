#!/usr/bin/expect -f

source ./_init.tcl

spawn promptctl ssh -i $ssh_id -p $ssh_port -o StrictHostKeyChecking=accept-new  $ssh_dest
ssh_login "$ssh_pw" "$cliprompt"
setup

test_security "$cliprompt"

run_prompt "$cmd" "$expected"
run_prompt "$cmd" "$expected"
run_prompt "$cmd" "$expected"

send "exit\r"
expect eof

########## Ensure Cleanup
spawn ssh -i $ssh_id -p $ssh_port -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no  $ssh_dest "ls -l /tmp | grep pcmd"
ssh_enter_pw "$ssh_pw"
test_cleanup
########################

## nested
spawn promptctl ssh -i $ssh_id -p $ssh_port -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no  $ssh_dest
ssh_login "$ssh_pw" "$cliprompt"

send "$shell\r"
run_prompt "$cmd" "$expected"
send "$shell\r"
run_prompt "$cmd" "$expected"
send "exit\r"
send "exit\r"

send "exit\r"
expect eof

########## Ensure Cleanup
spawn ssh -i $ssh_id -p $ssh_port -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no  $ssh_dest "ls -l /tmp | grep pcmd"
ssh_enter_pw "$ssh_pw"
test_cleanup
########################

### Remote cmd
spawn promptctl ssh -i $ssh_id -p $ssh_port -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no $ssh_dest "$cmd"
ssh_enter_pw "$ssh_pw"
expect "$expected"

### --remote
spawn $promptname {*}$promptargs --remote-dest $ssh_dest --remote-port $ssh_port
ssh_enter_pw "$ssh_pw"
expect "$expected"
