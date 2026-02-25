#!/bin/bash

set -e
set -o nounset

source ./_vars.sh

initialize() {
  mkdir -p $HOME_DIR
  mkdir -p $CONFIG_DIR
  mkdir -p $PROMPTS_DIR
  mkdir -p $INSTALL_DIR

}

create_config() {
  local shell=$1
  local bash_method=$2
  local channel=$3

  cat > $CONFIG_PATH << EOF
[[ssh]]
shell = "$shell"
channel = "$channel"
bash_method = "$bash_method"
EOF
}

$@
