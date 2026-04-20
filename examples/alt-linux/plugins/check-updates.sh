#!/usr/bin/env bash
# check-updates.sh — ashell update checker for ALT Linux (Sisyphus/p11)
#
# Output format: "<package> <old_version> -> <new_version>"
# One package per line — exactly what the ashell Updates module parser expects.
#
# Usage in config.toml:
#   check_cmd = "~/.config/ashell/check-updates.sh"

sudo apt-get dist-upgrade --simulate 2>/dev/null \
  | awk '/^Inst /{
      pkg = $2
      old = ""
      new = ""
      if (match($0, /\[([^]]+)\]/, a)) { split(a[1], v, ":"); old = v[1] }
      if (match($0, /\(([^ ]+)/, b)) { split(b[1], v, ":"); new = v[1] }
      if (old != "") print pkg, old, "->", new
  }'
