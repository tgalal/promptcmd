proc test_cleanup {} {

  expect {
    # Regex matching:
    #-re $cliprompt
    # Exact matching:
    #-exact $cliprompt
    # Default is glob matching:
    -re "(?i)(error|failed|permission denied|not found)" {
        puts stderr "ERROR detected in output"
        puts stderr "---- buffer ----"
        puts stderr $expect_out(buffer)
        puts stderr "----------------"
        exit 1
    }
    pcmd_* {
      puts stderr "ERROR: expected all tmp files to be cleanup"
      puts stderr "---- buffer ----"
      puts stderr $expect_out(buffer)
      puts stderr "----------------"
      exit 1
    }
  }

}

proc run_prompt {promptname expected_output} {
  run_cmd "$promptname" "$expected_output"
}

proc run_cmd {cmd expected_output} {
  send "$cmd\r"

  expect {
    -re "(?i)(error|failed|permission denied|not found)" {
        puts stderr "ERROR detected in output"
        puts stderr "---- buffer ----"
        puts stderr $expect_out(buffer)
        puts stderr "----------------"
        exit 1
    }
    $expected_output {
    }
    timeout {
      puts stderr "ERROR: expected command result not seen"
      exit 1
    }
  }
}

proc ssh_enter_pw {ssh_pw} {
  # Password login
  if {$ssh_pw ne ""} {
    expect {
      "password:" {
      }
      "Password for" {
      }
      timeout {
        puts stderr "ERROR: expected first password entry prompt but not seen"
        puts stderr "---- buffer ----"
        puts stderr $expect_out(buffer)
        puts stderr "----------------"
        exit 1
      }
    }
    send "$ssh_pw\r"
  }
}

proc ssh_login {ssh_pw cliprompt} {
  # Password login
  ssh_enter_pw "$ssh_pw"

  expect {
    # Regex matching:
    #-re $cliprompt
    # Exact matching:
    #-exact $cliprompt
    # Default is glob matching:
    -re "(?i)(error|failed|permission denied|not found)" {
        puts stderr "ERROR detected in output"
        puts stderr "---- buffer ----"
        puts stderr $expect_out(buffer)
        puts stderr "----------------"
        exit 1
    }
    -exact $cliprompt {
    }
    timeout {
      puts stderr "ERROR: expected login prompt not seen"
      puts stderr "---- buffer ----"
      puts stderr $expect_out(buffer)
      puts stderr "----------------"
      exit 1
    }
}
############### Login Completed
}

proc list_to_quoted_string {lst} {
    set parts [list]
    foreach item $lst {
        if {[string match "* *" $item]} {
            lappend parts "\"$item\""
        } else {
            lappend parts $item
        }
    }
    return [join $parts " "]
}

proc setup {} {
  send "echo -n 'testfile1content' > /tmp/testfile1\r"
  send "echo -n 'testfile2content' > /tmp/testfile2\r"
}

proc test_security {cliprompt} {

  ## security
  send "stat -c '%a' /tmp/pcmd_*\r"
  expect -exact "700"
  expect -exact $cliprompt

  send "stat -c '%a' /tmp/pcmd_*/*\r"
  # Expect the prompt back, capturing everything before it
  # expect -re {((?:.*\n)*.*)\n$cliprompt}
  # First, skip past the echoed command
  expect "\n"
  # Now capture everything up to the prompt
  expect -exact $cliprompt
  set output $expect_out(buffer)
  set output [string range $output 0 end-[string length $cliprompt]]
  # puts stderr "BUFFER: $output"
  # puts stderr "ENDBUFFER"

  set output [string map {\r ""} $output]
  regsub -all {\x1b\[[\x20-\x3f]*[\x40-\x7e]} $output "" output

  foreach line [split [string trim $output] "\n"] {
      set line [string trim $line]
      if {$line eq ""} continue
      # if {[string match "zsh*" $line]} continue
      if {![string is integer -strict $line]} continue
      if {$line ne "600"} {
          binary scan $line H* hex
          puts stderr "HEX: $hex"
          error "Expected 600, got: $line"
      }
  }
}
