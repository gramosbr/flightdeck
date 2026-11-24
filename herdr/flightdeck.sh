#!/bin/sh
# Runs the flightdeck command: the release's binary in bin/, else a dev build in tui/target/release/. Without either it
# exits quietly, since herdr runs it on events, where an error helps nobody; flightdeck doctor reports the missing binary.
dir=$(cd "$(dirname "$0")" && pwd)
FLIGHTDECK_HERDR_DIR=$dir
export FLIGHTDECK_HERDR_DIR
for binary in "$dir/bin/flightdeck" "$dir/tui/target/release/flightdeck"; do
  [ -x "$binary" ] && exec "$binary" "$@"
done
exit 0
