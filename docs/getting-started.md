# Getting started

Ten minutes from nothing to a flightdeck showing your own work.

## 1. What you need

- macOS or Linux (x86_64 or arm64).
- [herdr](https://herdr.dev) 0.9 or later with a Nerd Font, the Herdr desktop app, or both. flightdeck draws its
  spaces inside one of them, so this is the one thing you cannot skip.
- `gh`, signed in (`gh auth login`), and `jq`.

Everything else — tickets, CI, meetings, mail, deploys, the release monitor — is optional and comes from MCP
servers you already use. The first setup can leave all of them out.

## 2. Install

In Claude Code:

```
/plugin install flightdeck --marketplace gramosbr/flightdeck
/flightdeck:setup
```

With any other agent, in a shell:

```sh
curl -fsSL https://raw.githubusercontent.com/gramosbr/flightdeck/main/install.sh | sh
flightdeck setup
```

The shell installer checks the three things above before it downloads anything, and says what is missing.

Setup asks what you want and wires herdr, the Herdr app, or both. What flightdeck asks an agent to do (recaps, routines,
meeting briefs) runs on the agent you pick there. Notifications use a small notifier app on macOS, `notify-send` on
Linux.

## 3. Connect your work

```sh
flightdeck setup
```

Setup asks how far to go. **Just my pull requests, to-do and runbooks** needs nothing connected — GitHub comes
through `gh` — and is done in about a minute. The other answer walks through tickets, CI, meetings, mail, deploys
and the monitor, connecting each to a server you already use. Run `flightdeck setup` again whenever you want to add
a source.

Your work fills the spaces within a minute.

## 4. Move around

| Key | Space |
| --- | --- |
| `prefix+i` | Inbox — PRs, reviews, tickets, CI, previews |
| `prefix+d` | To-do |
| `prefix+u` | Runbook — the steps of the work in hand |
| `prefix+m` | Meetings |
| `prefix+o` | Monitor — what each release did to production |
| `prefix+y` | Routines |
| `prefix+shift+c` | Review — an agent's changes, commented line by line and sent back to it (pinned if you ask for it) |
| `prefix+space` | The palette: everything, searchable |

In every space: `j`/`k` move, `Enter` does the main action, `/` filters, `q` leaves. The row you are on shows its
actions with their keys, and the palette lists each space with the key that opens it.

In the Herdr app the same spaces are `⌘⌥1` to `⌘⌥7`, and Home shows which agents are waiting for you.

## 5. Day to day

- `flightdeck doctor` checks every part and names the fix for anything wrong.
- `/flightdeck` in Claude Code flips the settings worth changing often; everything else is in
  [configuration.md](configuration.md).
- In the Herdr app, **Flightdeck settings** at the foot of the sidebar has every option with a switch, a choice or a
  field, the same checks as `flightdeck doctor` under Health, and the version you run beside the published one, with a
  button that checks for a new one.
- flightdeck tells you when a new version is out; `flightdeck update` updates every part, then restarts the apps. In
  Claude Code alone, `/plugin update flightdeck@flightdeck` updates the plugin.
- Something not working? [troubleshooting.md](troubleshooting.md).
