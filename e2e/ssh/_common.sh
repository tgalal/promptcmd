logn() {
  local tag="$1"
  if [ -v 2 ]; then
      local message="$2"
      echo -n "[$tag] $message"
  else
    echo -n "$tag"
  fi
}

log() {
  local tag="$1"
  if [ -v 2 ]; then
      local message="$2"
      logn "$tag" "$message"
  else
    logn "$tag"
  fi
  echo
}

pause() {
  read -p "Press Enter to continue..." < /dev/tty
}

