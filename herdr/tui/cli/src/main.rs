//! flightdeck, the plugin's one binary: the space apps, the show view and the hub run as its subcommands, beside what the
//! plugin's scripts and herdr's events call.

mod act;
mod actions;
mod browse;
mod browser;
mod connect;
mod connector;
mod doctor;
mod find;
mod focus;
mod gui;
mod herdr_config;
mod home;
mod hub_env;
#[cfg(test)]
mod hub_restart;
mod launch;
mod mapper;
mod mcp;
mod options;
mod proc;
mod queue;
mod routine;
mod session;
mod setup;
mod shots;
mod space;
mod store;
mod update;
mod usage;

/// mimalloc hands freed pages back to the system soon after: the system allocator kept the tens of megabytes a show
/// picture's draw frees, so an idle pane held them.
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

const USAGE: &str = "flightdeck: the spaces' apps, the hub they share and the agents' tools.

    flightdeck inbox|runbook|todo|meetings|monitor|routines|review|mail
                                               a space's app (flightdeck space adds flightdeck-space, which update.sh looks for)
    flightdeck space <app>                        keeps that app running in its space; ctrl+c twice in a row leaves it
    flightdeck show <name>                        the view an agent session shows beside itself
    flightdeck palette                            the launcher herdr opens as a popup (prefix+space); PALETTE_PANE is the pane below
    flightdeck start-work [source]                the Start work form herdr opens as a popup (prefix+shift+s): a prompt, a
                                               ticket's key or a PR's URL becomes a worktree, a space and an agent
    flightdeck agents [answer|recap|snooze|seen|stop|fork|week]  every agent terminal: who needs you, answered from here
    flightdeck agents connect|disconnect [<agent>...]
                                               flightdeck's tools (flightdeck mcp) in each installed agent's MCP config
    flightdeck agents connected                   which agents have them, as JSON
    flightdeck usage [--json]                     every agent's limits with their reset, source and freshness, its account,
                                               and the last 7 days' tokens and estimated spend (a popup on a terminal)
    flightdeck agent id|argv ...                  an agent's id, or its command line to start, run, resume or fork (flightdeck agent for more)
    flightdeck search <query>                     what the palette finds, as the hub's JSON: items, the week's events, past sessions
    flightdeck mcp                                flightdeck's agent tools (show, search, changes, report_step) as a stdio MCP server
    flightdeck events [--since 30m|2h|1d|<epoch ms>] [--app inbox|monitor|routines]
                                               what changed in the shared data, newest first, as the hub's JSON (a day by default)
    flightdeck hub                                the background process the apps share; the first app that needs it starts it
    flightdeck hub ping                           exits 0 when the hub answers within 2 seconds
    flightdeck hub start                          starts the hub when none runs, prints {port, token, pid} for the Herdr app
    flightdeck hub stop                           stops the hub; nothing starts it again until flightdeck hub start
    flightdeck hub shown <view>                   the args of every drawn copy of a Herdr app view, as JSON
    flightdeck hub act review send|walkthrough|stop_walkthrough <JSON args>
                                               a remote desk's review action, handed to this machine's hub
    flightdeck app install|uninstall              the flightdeck layout, mod and keys in the Herdr app (flightdeck app for more)
    flightdeck update                             updates every part of flightdeck to the latest release, then restarts what runs it
    flightdeck update-apps [herdr folder]         installs the release's binary for the plugin's version
    flightdeck actions tab <cwd> <label> [key]   a tab in the actions space, its pane id printed
    flightdeck actions sidebar                    the actions space's sessions listed under its sidebar row
    flightdeck act <review|ci|rerun|plan|routine> <ref>
                                               what a notification's button does
    flightdeck notify [--system] <title> <body> [action] [<label>=<action> ...]
                                               a notification (the Herdr app's toast when it follows the hub, else
                                               the system's; --system: the system's); a click or a button does its action
    flightdeck focus <target>                     what a notification's click does (flightdeck focus for more)
    flightdeck setup-run <command> [args...]      start work's setup command, run as flightdeck's own work: the hub takes
                                               nothing only a person does from it or what it starts
    flightdeck setup                              picks the frontends, the default agent and the features, writes the config,
                                               wires herdr, the Herdr app and the other agents (/flightdeck:setup in Claude Code)
    flightdeck options [--json]                   every option section by section: its value, its default, what it takes
    flightdeck options set <key> <value>          sets one in ~/.config/flightdeck/config.json: on|off, one of a choice's
                                               values, a field's text or a number, checked first (a person at a terminal)
    flightdeck options seen [<key>...]            marks options as seen (every one without a key), and the last update shown
    flightdeck options new|ask                    what the last update brought, its new options with how to set them (ask:
                                               each one asked in turn, as herdr's popup after an update does)
    flightdeck options pending                    exits 0 while the last update, with something to say, was not shown
    flightdeck doctor [--all]                     checks every part of flightdeck: what is wrong, or with --all every check
    flightdeck doctor --json [--cached]           the same checks as data, with getting started and the agents (--cached:
                                               the last report when under ten minutes old)
    flightdeck browse ...                         a page in the side browser beside a pane (flightdeck browse for more)
    flightdeck browser ...                        the Chromium side browser: install, status, profiles (flightdeck browser for more)
    flightdeck shots ...                          screenshots at phone, tablet and desktop sizes (flightdeck shots for more)
    flightdeck store ...                          the documents the hub keeps and its leases, for the mod (flightdeck store for more)
    flightdeck routine run <slug>                 a routine run now and waited for: how it went, what it called, its output
    flightdeck resume [look|all|<terminal>|dismiss <terminal>]
                                               the agents a restart stopped, listed, or resumed where they were
                                               (look: notes what runs now first, as the hub does every minute)
    flightdeck queue [list|add <terminal> <text>|send <terminal> [all|<id>]|clear <terminal>|pick [terminal]]
                                               an agent's prompt queue (terminal: app:t3 or a herdr pane id)";

fn main() {
    flightdeck_core::answer_version("flightdeck");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some((command, rest)) = args.split_first() else { proc::fail(USAGE) };
    // Asking for help is not a mistake: it answers on stdout and exits 0, whole or for one command.
    if matches!(command.as_str(), "--help" | "-h" | "help") {
        println!("{USAGE}");
        return;
    }
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        usage_of(command);
        return;
    }
    match command.as_str() {
        "inbox" => app(flightdeck_inbox::run()),
        "start-work" => app(flightdeck_inbox::start_work(&rest.join(" "))),
        "runbook" => app(flightdeck_runbook::run()),
        "todo" => app(flightdeck_todo::run()),
        "meetings" => app(flightdeck_meetings::run()),
        "monitor" => app(flightdeck_monitor::run()),
        "routines" => app(flightdeck_routines::run()),
        "review" => app(flightdeck_review::run()),
        "mail" => app(flightdeck_mail::run()),
        "show" => app(flightdeck_show::run(rest.first().map_or("show", String::as_str))),
        "palette" => app(flightdeck_palette::run()),
        "agents" if rest.first().is_some_and(|c| ["connect", "disconnect", "connected"].contains(&c.as_str())) => connect::main(rest),
        "agents" => flightdeck_agents::cli::main(rest),
        "session" => session::main(rest),
        "agent" => match launch::run(rest) {
            Ok(line) => println!("{line}"),
            Err(why) => proc::fail(&why),
        },
        "search" => find::search(rest),
        "mcp" => mcp::main(rest),
        "connector" => connector::main(rest),
        "events" => find::events(rest),
        "hub" if rest.is_empty() => gui::hub(),
        "hub" if rest == ["start"] => gui::hub_start(),
        "hub" if rest == ["stop"] => gui::hub_stop(),
        "hub" if rest.len() == 2 && rest[0] == "shown" => gui::hub_shown(&rest[1]),
        "hub" if rest.first().is_some_and(|a| a == "act") => act::hub_act(&rest[1..]),
        "app" => gui::main(rest),
        "hub" if rest == ["ping"] => std::process::exit(if ping(Duration::from_secs(2)) { 0 } else { 1 }),
        "update" if rest.is_empty() => {
            let script = flightdeck_core::files::herdr_dir().join("update.sh");
            let status = std::process::Command::new("sh").arg(&script).env("PATH", proc::path()).status();
            std::process::exit(status.ok().and_then(|s| s.code()).unwrap_or(1));
        }
        "update-apps" => update::main(rest),
        "actions" => actions::main(rest),
        "act" => act::main(rest),
        "notify" => notify(rest),
        "focus" => focus::main(rest),
        "setup-run" => setup_run(rest),
        "doctor" => doctor::main(rest),
        "setup" => setup::main(rest),
        "options" => options::main(rest),
        "space" => space::main(rest),
        "browse" => browse::main(rest),
        "browser" => browser::main(rest),
        "shots" => shots::main(rest),
        "store" => store::main(rest),
        "routine" => routine::main(rest),
        "resume" => resume(rest),
        "queue" => queue::main(rest),
        "usage" => usage::main(rest),
        _ => proc::fail(USAGE),
    }
}

/// What USAGE says about one command, on stdout; the whole of it when it names none.
fn usage_of(command: &str) {
    let head = format!("    flightdeck {command}");
    let mut lines = Vec::new();
    for line in USAGE.lines() {
        let names = line.starts_with(&head) && line[head.len()..].starts_with([' ', '|']);
        // A command's text runs on over the indented lines under it.
        let continues = !lines.is_empty() && line.starts_with("     ") && !line.trim_start().starts_with("flightdeck ");
        if names || continues {
            lines.push(line);
        } else if !lines.is_empty() && !line.trim().is_empty() {
            break;
        }
    }
    if lines.is_empty() {
        println!("{USAGE}");
        return;
    }
    // USAGE indents every line by four spaces; drop those so one command's help reads on its own.
    for line in lines {
        println!("{}", line.strip_prefix("    ").unwrap_or(line));
    }
}

/// The agents a restart stopped: listed as JSON, all resumed, one resumed, or one left stopped.
fn resume(args: &[String]) {
    use flightdeck_core::resume;
    let done = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        [] => {
            println!("{}", serde_json::Value::from(resume::lost()));
            return;
        }
        ["look"] => {
            let (record, newly) = resume::remember();
            println!("{}", serde_json::json!({"record": record, "newly_lost": newly}));
            return;
        }
        ["all"] => resume::resume_all(),
        ["dismiss", handle] => {
            resume::dismiss(handle);
            Ok(String::new())
        }
        [handle] => resume::resume(handle),
        _ => Err("flightdeck resume [look|all|<terminal>|dismiss <terminal>]".into()),
    };
    match done {
        Ok(note) if !note.is_empty() => println!("{note}"),
        Ok(_) => {}
        Err(err) => proc::fail(&err),
    }
}

/// A system notification with the flightdeck icon: a click does the action, each `<label>=<action>` button its own. Any
/// process may run this, so a button's act becomes the row it is about, where the person acts themselves.
fn notify(args: &[String]) {
    let system = args.first().is_some_and(|a| a == "--system");
    let args = if system { &args[1..] } else { args };
    let [title, body, rest @ ..] = args else { proc::fail("flightdeck notify [--system] <title> <body> [action] [<label>=<action> ...]") };
    let action = no_act(rest.first().map_or("", String::as_str));
    let buttons: Vec<String> = rest
        .get(1..)
        .unwrap_or_default()
        .iter()
        .filter_map(|b| b.split_once('=').map(|(label, target)| (label, no_act(target))).filter(|(_, t)| !t.is_empty()).map(|(l, t)| format!("{l}={t}")))
        .collect();
    if system { flightdeck_core::notify::post_system(title, body, &action, &buttons) } else { flightdeck_core::notify::post(title, body, &action, &buttons) }
}

/// A target with its act, if any, turned into the row it is about: the hub runs an act only from a person's own click.
fn no_act(target: &str) -> String {
    let bare = target.strip_prefix(flightdeck_core::notify::IN_APP).unwrap_or(target);
    if bare.starts_with("act:") { flightdeck_core::notify::to_view(target) } else { target.to_string() }
}

/// Runs a start's setup command under this process and exits as it did: the hub reads the line of parents, so nothing
/// the command or its children ask for passes for the person's.
fn setup_run(args: &[String]) {
    let [program, rest @ ..] = args else { proc::fail("flightdeck setup-run <command> [args...]") };
    let status = std::process::Command::new(program).args(rest).status();
    let code = match status {
        Ok(s) => s.code().unwrap_or_else(|| 128 + std::os::unix::process::ExitStatusExt::signal(&s).unwrap_or(1)),
        Err(e) => proc::fail(&format!("flightdeck setup-run: {program}: {e}")),
    };
    std::process::exit(code);
}

/// An app's end: an error that stopped it is printed, and `flightdeck space` starts it again.
fn app(ended: std::io::Result<()>) {
    if let Err(error) = ended {
        proc::fail(&format!("flightdeck: {error}"));
    }
}

/// Whether the hub answers a ping on its socket in time; one that is not running is not started.
fn ping(timeout: Duration) -> bool {
    let Ok(mut stream) = UnixStream::connect(flightdeck_core::hub::socket()) else { return false };
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();
    if writeln!(stream, "{{\"op\": \"ping\"}}").is_err() {
        return false;
    }
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).is_ok() && serde_json::from_str::<serde_json::Value>(&line).is_ok_and(|answer| answer["ok"] == true)
}
