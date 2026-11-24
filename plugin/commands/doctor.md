---
description: Check every part of flightdeck and fix what is broken
---

Run `FLIGHTDECK_APP_WAIT=0 sh ~/.config/flightdeck/herdr/app.sh doctor` (when that link is missing, neither the herdr plugin nor the Herdr app half is set up: say so and suggest `/flightdeck:setup`). When it prints `no flightdeck binary for this machine yet` instead of checks, the `flightdeck` binary is missing and could not be fetched: say so, and suggest `sh ~/.config/flightdeck/herdr/fetch-apps.sh` (its error says why) or `cargo build --release` in `~/.config/flightdeck/herdr/tui`.

Reply with the `fail` and `warn` lines only, each as one short line: what is wrong and the fix the line names. When everything is `ok`, say so in one line.

Then offer to apply the fixes that only touch flightdeck's own pieces: running `ensure-spaces.sh`, `fetch-apps.sh`, `prune-browsers.sh`, `update.sh`, `notifier/build.sh`, `flightdeck.sh app install` (the Herdr app's layout and mod), `flightdeck.sh agents connect <agent>` (flightdeck's tools in an agent's MCP config, backed up first), `pkill -f '(^|/)flightdeck hub$'`, or sending `/reload-plugins` to an idle Claude session: `herdr agent prompt <pane> /reload-plugins` in herdr, `hc send <terminal> /reload-plugins` in the Herdr app. Apply them only once I agree. Never edit `~/.claude/settings.json`, the herdr config or the Herdr app's own files (`keys.json`, `settings.json`) yourself: print the change for me to make.
