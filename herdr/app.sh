#!/bin/sh
# Runs one flightdeck app as `flightdeck <app> [args]`: the release's binary in bin/, else a dev build in tui/target/release/.
# When neither is there it fetches the release's binary first, and otherwise says so and waits (FLIGHTDECK_APP_WAIT seconds,
# 30 by default) before it fails.
PATH="/opt/homebrew/bin:/usr/local/bin:$HOME/.local/bin:$PATH"
cd "$(dirname "$0")" || exit 1
name=$1
shift
FLIGHTDECK_HERDR_DIR=$(pwd)
export FLIGHTDECK_HERDR_DIR

# By its full path, which the hub looks for to restart the apps on a new binary.
for dir in bin tui/target/release; do
  [ -x "$dir/flightdeck" ] && exec "$FLIGHTDECK_HERDR_DIR/$dir/flightdeck" "$name" "$@"
done
sh fetch-apps.sh && [ -x bin/flightdeck ] && exec "$FLIGHTDECK_HERDR_DIR/bin/flightdeck" "$name" "$@"
echo "flightdeck: no flightdeck binary for this machine yet; run flightdeck doctor to see why" >&2
sleep "${FLIGHTDECK_APP_WAIT:-5}"
exit 1
