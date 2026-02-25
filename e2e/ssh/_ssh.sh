#!/bin/bash

set -e
set -o nounset

source ./_vars.sh

initialize() {
  mkdir -p $SSH_DIR
  chmod 700 $SSH_DIR
  keygen
}

keygen() {
  echo "Setting up SSH Keys"
  ssh-keygen -t ed25519 -f $SSH_IDENTITY -N "" > /dev/null
}

copy_id() {
  local ssh_dest=$1
  local port=$2
  local identity=$SSH_IDENTITY
  local password=$SSH_PASSWORD

  echo "Copying SSH Identity to $ssh_dest"

expect <<EOF
spawn ssh-copy-id -p $port -i $identity $ssh_dest

# Wait for password prompt and send password
expect "password:"
send "$password\r"

expect eof
EOF
}

$@
