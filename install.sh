#!/bin/sh
# Installs flightdeck without Claude Code, for any agent herdr or the Herdr app runs:
#
#   curl -fsSL https://raw.githubusercontent.com/gramosbr/flightdeck/main/install.sh | sh
#
# It puts the release's scripts and `flightdeck` binary for this machine in ~/.local/share/flightdeck (FLIGHTDECK_HOME), links
# ~/.local/bin/flightdeck and ~/.config/flightdeck/herdr to them, then runs `flightdeck setup`. Run it again to update.
# FLIGHTDECK_VERSION picks a version (the latest by default), FLIGHTDECK_NO_SETUP=1 skips the setup, FLIGHTDECK_NO_CHECKS=1
# the prerequisite checks.
set -eu

repo=${FLIGHTDECK_REPO:-gramosbr/flightdeck}
releases=${FLIGHTDECK_RELEASES_URL:-https://github.com/$repo/releases/download}
sources=${FLIGHTDECK_SOURCES_URL:-https://github.com/$repo/archive/refs/tags}
dest=${FLIGHTDECK_HOME:-$HOME/.local/share/flightdeck}
bindir=${FLIGHTDECK_BIN_DIR:-$HOME/.local/bin}

say() { printf 'flightdeck: %s\n' "$*"; }
die() { printf 'flightdeck: %s\n' "$*" >&2; exit 1; }

command -v curl >/dev/null 2>&1 || die "curl is needed"
command -v tar >/dev/null 2>&1 || die "tar is needed"

case "$(uname -s) $(uname -m)" in
  "Darwin arm64") target=aarch64-apple-darwin ;;
  "Darwin x86_64") target=x86_64-apple-darwin ;;
  "Linux x86_64") target=x86_64-unknown-linux-gnu ;;
  "Linux aarch64" | "Linux arm64") target=aarch64-unknown-linux-gnu ;;
  *) die "no release is built for $(uname -s) $(uname -m); build it from source: see https://github.com/$repo#develop" ;;
esac

# What flightdeck needs, checked before anything is downloaded: flightdeck runs inside herdr or the Herdr app, so without
# one of them there is nothing for it to run in, and the setup at the end would stop anyway.
if [ "${FLIGHTDECK_NO_CHECKS:-}" != 1 ]; then
  missing=""
  command -v jq >/dev/null 2>&1 || missing="$missing
  jq           brew install jq, or apt install jq"
  if ! command -v gh >/dev/null 2>&1; then
    missing="$missing
  gh           brew install gh, or see https://cli.github.com; then gh auth login"
  elif ! gh auth status >/dev/null 2>&1; then
    missing="$missing
  gh signed in gh auth login"
  fi
  [ -n "$missing" ] && printf 'flightdeck needs:%b

' "$missing" >&2
  if ! command -v herdr >/dev/null 2>&1 && [ ! -x "$HOME/.local/bin/herdr-app" ]; then
    die "flightdeck runs in herdr (the terminal) or the Herdr app (the window), and neither is installed.
Install one from https://herdr.dev, then run this again."
  fi
  [ -n "$missing" ] && die "install what is missing above, then run this again."
fi

version=${FLIGHTDECK_VERSION:-}
if [ -z "$version" ]; then
  # The latest release's page redirects to its tag.
  latest=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest") || die "could not reach GitHub"
  version=${latest##*/v}
  case "$version" in [0-9]*) ;; *) die "no release found at https://github.com/$repo/releases" ;; esac
fi
version=${version#v}

if [ "$(cat "$dest/herdr/bin/.version" 2>/dev/null)" = "$version" ] && [ -x "$dest/herdr/bin/flightdeck" ]; then
  say "flightdeck $version is already installed in $dest"
else
  mkdir -p "$(dirname "$dest")"
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/flightdeck-install.XXXXXX")
  trap 'rm -rf "$tmp"' EXIT
  trap 'exit 1' INT TERM HUP

  file=flightdeck-apps-$version-$target.tar.gz
  say "downloading flightdeck $version for $target"
  curl -fsSL --connect-timeout 10 --max-time 300 -o "$tmp/$file" "$releases/v$version/$file" || die "no $file in release v$version"
  curl -fsSL --connect-timeout 10 --max-time 60 -o "$tmp/$file.sha256" "$releases/v$version/$file.sha256" || die "no checksum for $file"
  want=$(cut -d ' ' -f 1 "$tmp/$file.sha256")
  got=$({ sha256sum "$tmp/$file" 2>/dev/null || shasum -a 256 "$tmp/$file"; } | cut -d ' ' -f 1)
  [ -n "$want" ] && [ "$want" = "$got" ] || die "checksum mismatch for $file"
  # The scripts of the same version, from its tagged sources: everything but the Rust sources.
  curl -fsSL --connect-timeout 10 --max-time 300 -o "$tmp/src.tar.gz" "$sources/v$version.tar.gz" || die "could not download the v$version sources"

  mkdir "$tmp/src" "$tmp/new"
  tar -xzf "$tmp/src.tar.gz" -C "$tmp/src" || die "unreadable sources archive"
  top=$(find "$tmp/src" -mindepth 1 -maxdepth 1 -type d | head -n 1)
  [ -f "$top/herdr/herdr-plugin.toml" ] || die "the v$version sources hold no herdr/ folder"
  cp -R "$top/herdr" "$tmp/new/herdr"
  rm -rf "$tmp/new/herdr/tui"
  [ -d "$top/plugin/skills" ] && mkdir -p "$tmp/new/plugin" && cp -R "$top/plugin/skills" "$tmp/new/plugin/skills"
  [ -d "$top/codex" ] && cp -R "$top/codex" "$tmp/new/codex"
  [ -f "$top/install.sh" ] && cp "$top/install.sh" "$tmp/new/install.sh"
  mkdir "$tmp/new/herdr/bin"
  tar -xzf "$tmp/$file" -C "$tmp/new/herdr/bin" || die "unreadable archive $file"
  [ -f "$tmp/new/herdr/bin/flightdeck" ] || die "no flightdeck binary in $file"
  chmod +x "$tmp/new/herdr/bin/flightdeck"
  printf '%s\n' "$version" >"$tmp/new/herdr/bin/.version"
  # What update.sh looks for to update this install by running install.sh again.
  printf '%s\n' "$version" >"$tmp/new/.flightdeck-install"

  # Swapped in by renames, so a running hub or app keeps a whole folder under it.
  rm -rf "$dest.old"
  [ -d "$dest" ] && mv "$dest" "$dest.old"
  mv "$tmp/new" "$dest"
  rm -rf "$dest.old"
  say "installed flightdeck $version in $dest"
fi

mkdir -p "$bindir"
ln -sfn "$dest/herdr/bin/flightdeck" "$bindir/flightdeck"
say "linked $bindir/flightdeck"
# The scripts' link, unless herdr's plugin or Claude Code's owns it (their scripts keep it on their own copy).
link=$HOME/.config/flightdeck/herdr
linked=$(readlink "$link" 2>/dev/null || true)
[ -e "$link" ] || linked=""
if [ -d "$link" ] && [ ! -L "$link" ]; then
  aside=$link.$(date +%Y%m%d-%H%M%S)
  mv "$link" "$aside"
  say "moved the folder at $link aside to $aside"
fi
case "$linked" in
  "" | "$dest/herdr")
    mkdir -p "$(dirname "$link")"
    ln -sfn "$dest/herdr" "$link"
    ;;
  */plugins/marketplaces/*)
    say "$link stays on the Claude Code plugin's copy; to use this install's instead: ln -sfn $dest/herdr $link"
    ;;
  *) say "$link stays on $linked (herdr's plugin)" ;;
esac
case ":$PATH:" in
  *":$bindir:"*) ;;
  *) say "add $bindir to your PATH to run flightdeck by name" ;;
esac

if [ "${FLIGHTDECK_NO_SETUP:-}" != 1 ]; then
  if [ -r /dev/tty ] && (exec </dev/tty) 2>/dev/null; then
    "$bindir/flightdeck" setup </dev/tty
  else
    say "next: flightdeck setup"
  fi
fi
