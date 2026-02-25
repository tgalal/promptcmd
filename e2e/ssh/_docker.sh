#!/bin/bash

set -e
set -o nounset

start() {
  local name=$1
  local port=$2
  echo "starting docker container: $name"
  docker run --rm -d -p $port:22 --name $name $name > /dev/null
}

stop() {
  local name=$1
  echo "stopping docker container: $name"
  docker stop $name > /dev/null
}

start_all() {
  start promptcmd-tests-debian-bookworm 2222
}

stop_all() {
  stop promptcmd-tests-debian-bookworm
}

$@
