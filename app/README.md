<!-- A copy of ../README.md, written by scripts/app-readme.sh: edit that one. -->

<p align="center"><img src="../docs/logo.svg" width="96" alt="flightdeck logo"></p>

<h1 align="center">flightdeck</h1>

<p align="center"><b>See every coding agent you are running, and what it is working on.</b><br>
Claude Code, Codex and the other agents Herdr runs, each shown with its ticket, pull request, CI run and deploy,<br>
and which of them are waiting for an answer. Your reviews, mail, runbooks, meetings and releases are one key away.</p>

<p align="center"><a href="https://gramosbr.github.io/flightdeck/"><b>Website</b></a> ·
<a href="https://gramosbr.github.io/flightdeck/desktop.html">Herdr APP</a> ·
<a href="https://gramosbr.github.io/flightdeck/terminal.html">Herdr TUI</a> ·
<a href="https://github.com/gramosbr/flightdeck/releases">Changelog</a></p>

<p align="center">flightdeck is a fork of <a href="https://github.com/quaresc/cockpit">cockpit</a> (MIT).</p>

<p align="center"><img src="../app/preview.png" width="100%" alt="flightdeck in the Herdr app: an agent with its page in a browser beside it, its session's links, and the inbox, mail, runbook and meetings views"></p>

## Install

You need macOS or Linux, [herdr](https://herdr.dev) 0.9+ or the Herdr app, `gh` signed in, and `jq`.

**In Claude Code**

```
/plugin install flightdeck --marketplace gramosbr/flightdeck
/flightdeck:setup
```

**With any other agent**, in a shell

```sh
curl -fsSL https://raw.githubusercontent.com/gramosbr/flightdeck/main/install.sh | sh
flightdeck setup
```

> [!NOTE]
> In the Herdr app, the gallery installs the layout in [`app/`](../app/). It shows flightdeck's data, so install flightdeck
> as above too.

`flightdeck update` updates every part later; flightdeck also tells you when a new version is out, and after an update asks
once about the options it brought (`flightdeck options` lists them all). In the Herdr app, **Flightdeck settings** at the
foot of the sidebar has every option, the version you run and the published one, and a button that checks for a new one.

<details>
<summary><b>Questions</b></summary>

- **Do I need API tokens?** No. flightdeck reads what your agents already reach through MCP, and GitHub through `gh`.
- **What does it cost?** Nothing. Work it asks an agent to do (recaps, routines, briefs) is billed to that agent.
- **Herdr app or herdr?** Either, or both. Setup asks.
- **Is my data sent anywhere?** No. It stays on your machine.

</details>

<details>
<summary><b>Agents and tools</b></summary>

- Works with every agent Herdr runs.
- Tickets, CI, meetings, mail and deploys come from MCP servers you already use. Setup knows Linear, Jira, GitHub,
  GitLab, CircleCI, Datadog, Sentry, Gmail and Google Calendar, and your agent maps any other.
- Agents draw charts, tables and diagrams beside their session with the `show` tool.
- Agents check pages in a browser beside them: terminal-browser in a pane, or in the Herdr app on macOS a Chromium
  window in the side panel, with a browser per agent session, DevTools, and elements you point at for that agent.

</details>

## More

[Getting started](../docs/getting-started.md) · [Configuration](../docs/configuration.md) · [Show blocks](../docs/show.md) ·
[The Chromium side browser](../docs/browser.md) ·
[Troubleshooting](../docs/troubleshooting.md) · [Changelog](https://github.com/gramosbr/flightdeck/releases) ·
[Contributing](../CONTRIBUTING.md) · MIT ([notices](../THIRD_PARTY_NOTICES.md))
