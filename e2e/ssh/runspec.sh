#!/bin/bash

set -e
set -o nounset

source ./_vars.sh
source ./_common.sh

initialize_environment() {
  rm -rf $STORAGE_DIR
  mkdir $STORAGE_DIR

  ./_promptcmd.sh initialize
  ./_ssh.sh initialize
}

_server() {
  local server=$1
  local pre=$2
  local post=$3

  if [ -v 4 ]; then
    local filter_shell=$4
    local filter_bash_method=$5
    local filter_channel=$6
  else
    local filter_shell=""
    local filter_bash_method=""
    local filter_channel=""
  fi


  # echo "should filter for $shell $bash_method $channel"

  test_desc=$(echo "$server" | jq -r '.desc')
  ssh_dest=$(echo "$server" | jq -r '.dest')
  ssh_port=$(echo "$server" | jq -r '.port')
  ssh_password=$(echo "$server" | jq -r '.password')
  jdest=$(echo "$server" | jq -r '.jdest')
  jport=$(echo "$server" | jq -r '.jport')
  jpassword=$(echo "$server" | jq -r '.jpassword')

  log "Test Server ${ssh_dest}" ""

  echo "$server" | jq -r '.shells[] | "\(.name)|\(.prompt)|\(.desc)|\(.channels)|\(.nested)|\(.bash_method)"' | while IFS='|' read -r shell_name shell_prompt test_desc channels nested bash_method; do
      # echo "  Shell name: $shell_name"
      # echo "  Shell prompt: $shell_prompt"
      # echo "  Default shell: $default_shell"
      # echo "  ---"
      #

      if [ $bash_method == "null" ]; then
        bash_method="rc"
      fi

      if [[ -n "$filter_bash_method" && "$filter_bash_method" != "$bash_method"  ]]; then
        echo "Skipping bash_method: $bash_method"
        continue
      fi

      if [[ -n "$filter_shell" && "$filter_shell" != "$shell_name"  ]]; then
        echo "Skipping shell: $shell"
        continue
      fi

      echo "$channels" | jq -r ".[]" | while read -r channel; do
        if [[ -n "$filter_channel" && "$filter_channel" != "$channel"  ]]; then
          echo "Skipping channel: $channel"
          continue
        fi
        # if [ $jdest != "null" ]; then
        #   logn "Test" "name: jump_dest, jdest: ${jdest}:${jport}"
        # ./dispatcher.sh jump_server "$ssh_dest" "$ssh_port" "$ssh_password" "$jdest" "$jport" "$jpassword" "$shell_name" $bash_method $channel "${shell_prompt}"
        # fi
        logn "Test" "name: dummy, shell: ${shell_name}, bash_method: ${bash_method} channel: $channel "
        ./dispatcher.sh basic_tests $ssh_dest $ssh_port "$ssh_password" "$shell_name" "$nested" "$bash_method" "$channel" "${shell_prompt}"
        log " [ok]"

      done
  done
}

_exit() {
  local postcmdlist="$1"

  echo "$postcmdlist" | jq -rc ".[]" | while read -r postcmd; do
    $postcmd
  done
}

_spec() {
  local specfile=$1
  if [ -v 2 ]; then
    local shell=$2
    local bash_method=$3
    local channel=$4
  else
    local shell=""
    local bash_method=""
    local channel=""
  fi

  initialize_environment

  # pre scripts
  jq -rc ".pre[]" ${specfile} | while read -r precmd; do
    $precmd
  done

  # post scripts
  postcmd=$(jq -rc ".post" ${specfile})
  trap '_exit "$postcmd"' EXIT

  echo
  jq -c '.servers[]' ${specfile} | while read -r server; do
  _server "$server" "" "" "$shell" "$bash_method" "$channel"

  done
}

_spec "$@"
