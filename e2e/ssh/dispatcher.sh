#!/bin/bash

set -e
set -o nounset

source ./_vars.sh

EXECUTOR=./tcl/basic_test.tcl

dispatch() {
# usage: "Test Desc" "cliprompt"
# script.tcl ctl ssh_dest ssh_port ssh_identity ssh_password cliprompt command expected
local testname=$1
local ssh_dest=$2
local ssh_port=$3
local pw=$4
local shell=$5
local bash_method=$6
local channel=$7
local cliprompt=$8
local promptname=$9
local expected=${10}
local prescript=${11}
local postscript=${12}

./_promptcmd.sh create_config $shell $bash_method $channel

# echo -n "$description "

$EXECUTOR $PROMPTCTL $ssh_dest $ssh_port $SSH_IDENTITY "$pw" "$cliprompt" "$promptname" "$expected" "$prescript" "$postscript" > /dev/null

# echo "[Ok]"
}

run_tcl_test() {
  local testname="$1"
  shift
  (cd ./tcl && ./${testname}.tcl "$@" > /dev/null)
}

jump_server() {
  local dest1=$1
  local port1=$2
  local pw1=$3
  local dest2=$4
  local port2=$5
  local pw2=$6
  local shell=$7
  local bash_method=$8
  local channel=$9
  local cliprompt=${10}

  ./_promptcmd.sh create_config "$shell" "$bash_method" "$channel"

  $PROMPTCTL import -fep dummy - > /dev/null << EOF
---
input:
  schema:
    input1: string
    input2: string
---
Input1: {{input1}} Input2: {{input2}} Data1: {{exec "cat" "/tmp/testfile1"}} Data2: {{cat "/tmp/testfile2"}}
EOF
  run_tcl_test "jump_server" "$SSH_IDENTITY" \
    $dest1 $port1 $pw1 $dest2 $port2 $pw2 $cliprompt \
    "Input1: one two three Input2: four five six Data1: testfile1content Data2: testfile2content" \
    "dummy" \
    --input1 'one two three' --input2 'four five six' --render
}

basic_tests() {
  # usage: dummy ssh_dest pw shell channel cliprompt
  local ssh_dest=$1
  local ssh_port=$2
  local pw=$3
  local shell=$4
  local real_shell=$5
  local bash_method=$6
  local channel=$7
  local cliprompt=$8
  local description="Test dummy prompt with $shell:$channel on $ssh_dest"

  ./_promptcmd.sh create_config $shell $bash_method $channel

  $PROMPTCTL import -fep dummy - > /dev/null << EOF
---
input:
  schema:
    input1: string
    input2: string
---
Input1: {{input1}} Input2: {{input2}} Data1: {{exec "cat" "/tmp/testfile1"}} Data2: {{cat "/tmp/testfile2"}}
EOF

  run_tcl_test "basic_tests" "$ssh_dest" "$ssh_port" "$SSH_IDENTITY" "$pw" \
    "$cliprompt" \
    "$real_shell" \
    "Input1: one two three Input2: four five six Data1: testfile1content Data2: testfile2content" \
    "dummy" \
    --input1 'one two three' --input2 'four five six' --render
}

nested() {
  # usage: dummy ssh_dest pw shell channel cliprompt
  local ssh_dest=$1
  local ssh_port=$2
  local pw=$3
  local shell=$4
  local bash_method=$5
  local channel=$6
  local cliprompt=$7
  local real_shell=$8 # because $shell maybe a special key like "auto, bashposix, ..etc"
  local description="Test dummy prompt with $shell:$channel on $ssh_dest"

  $PROMPTCTL import -fep dummy - > /dev/null << EOF
Dummy
EOF

  dispatch "nested" $ssh_dest $ssh_port "$pw" $shell $bash_method $channel "$cliprompt" "dummy --render" "Dummy" "$real_shell" "exit"
}

"$@"
