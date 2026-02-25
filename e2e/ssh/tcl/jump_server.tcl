#!/usr/bin/expect -f
source ./_common.tcl

set timeout 60
set ssh_id [lindex $argv 0]
set dest1 [lindex $argv 1]
set port1 [lindex $argv 2]
set pw1 [lindex $argv 3]
set dest2 [lindex $argv 4]
set port2 [lindex $argv 5]
set pw2 [lindex $argv 6]
set cliprompt [lindex $argv 7]
set expected [lindex $argv 8]
set promptname [lindex $argv 9]
set promptargs [lrange $argv 10 end]
set promptargs_str [list_to_quoted_string $promptargs]

set cmd "$promptname $promptargs_str"

spawn promptctl ssh -i $ssh_id  -J $dest1:$port1 -o StrictHostKeyChecking=accept-new -p $port2 $dest2
puts stderr "AHOOO $cliprompt"

ssh_enter_pw "$pw2"
ssh_login "$pw1" "$cliprompt"

run_prompt "$cmd" "$expected"

send "exit\r"
expect eof

