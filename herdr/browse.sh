#!/bin/sh
# Opens a URL beside an agent session in the side browser (terminal-browser in a pane, or flightdeck's Chromium window in
# the Herdr app), else in the default browser.
#   browse.sh [--size <fraction>] [--reload] <url> [<session id>]          the session's pane, else the calling pane
#   browse.sh --beside [<session id>]                                      the browser beside it, as JSON
#   browse.sh --external [<session id>]                                    its page, in the default browser
#   browse.sh action [--session <session id>] [--] <command>               an agent-browser command on the page beside it
PATH="/opt/homebrew/bin:/usr/local/bin:$HOME/.local/bin:$PATH"
dir=$(dirname "$0")
for binary in "$dir/bin/flightdeck" "$dir/tui/target/release/flightdeck"; do
  [ -x "$binary" ] && exec "$binary" browse "$@"
done
echo "flightdeck: no flightdeck binary for this machine yet; run flightdeck doctor to see why" >&2
exit 1
