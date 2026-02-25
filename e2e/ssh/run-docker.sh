#!/bin/bash

set -e
set -o nounset

source ./_common.sh

declare -A CONTAINERS
CONTAINERS["promptcmd-tests-tinycore"]="./spec/docker-tinycore.json"
CONTAINERS["promptcmd-tests-alpine-nofwd"]="./spec/docker-alpine-nofwd.json"
CONTAINERS["promptcmd-tests-alpine"]="./spec/docker.json"
CONTAINERS["promptcmd-tests-fedora"]="./spec/docker-fedora.json"
CONTAINERS["promptcmd-tests-ubuntu-jammy"]="./spec/docker.json"
CONTAINERS["promptcmd-tests-debian-bullseye-slim"]="./spec/docker.json"
CONTAINERS["promptcmd-tests-debian-bookworm-slim"]="./spec/docker.json"

tests_mode=0
active_container=""

teardown() {
  if [[ -n $active_container ]]; then
    log "Container $active_container" "stopping"
    docker stop ${active_container}
    active_container=""
  fi
}

setup() {
  local container=$1
  log "Container $container" "starting"
  docker run --rm -d -p 2222:22 --name "$container" "$container"
  active_container=$container

  if [ "$tests_mode" -eq "0" ]; then
    pause
  fi

  trap "teardown" EXIT
}

run_for_container() {
  local container=$1
  local spec=$2
  local shell=$3
  local bash_method=$4
  local channel=$5
  setup "$container"
  ./runspec.sh "$spec" $shell $bash_method $channel
  teardown

}

all() {
  for container in "${!CONTAINERS[@]}"; do
    spec=${CONTAINERS[$container]}
    run_for_container "$container" "$spec" "" "" ""
  done
}

container=$1

if [ $container = "setup" ]; then
  setup "$2"
  exit
fi

# if {$output eq "Basic Test\n"} {
if [ $container = "all" ]; then
  tests_mode=1
  all
else
  tests_mode=1
  if [ -v 2 ]; then
    shell=$2
    bash_method=$3
    channel=$4
  else
    shell=""
    bash_method=""
    channel=""
  fi

  spec=${CONTAINERS[$container]}
  run_for_container $container $spec "$shell" "$bash_method" "$channel"
fi
# "$@"
