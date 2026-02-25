source ./_common.tcl

set timeout 4
set ssh_dest [lindex $argv 0]
set ssh_port [lindex $argv 1]
set ssh_id [lindex $argv 2]
set ssh_pw [lindex $argv 3]
set cliprompt [lindex $argv 4]
set shell [lindex $argv 5]
set expected [lindex $argv 6]
set promptname [lindex $argv 7]
set promptargs [lrange $argv 8 end]
set promptargs_str [list_to_quoted_string $promptargs]

set cmd "$promptname $promptargs_str"
