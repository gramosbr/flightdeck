//! The plugin's shell scripts that start the binary: app.sh and flightdeck.sh find it, fetch-apps.sh installs it from a
//! release (served here by a small HTTP server) or has a binary do it, update.sh stops the spaces' supervisors, their
//! apps and the hub, pin-spaces.sh orders the spaces, browse.sh and shots.sh hand their arguments to the flightdeck command,
//! palette.sh opens the palette popup over the pane in view, and whats-new.sh the popup after an update.

mod common;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use common::{Sandbox, executable, scripts, stderr, stdout};

// ---------------------------------------------------------------------------
// app.sh

/// app.sh run on an app `demo` with these flightdeck binaries in bin/ and tui/target/release/, and what it printed.
fn run_app(release: Option<&str>, dev: Option<&str>, fetch: &str) -> (Option<i32>, String, String) {
    let sandbox = Sandbox::new();
    sandbox.copy_script("app.sh");
    executable(&sandbox.root.join("herdr/fetch-apps.sh"), fetch);
    for (dir, body) in [("bin", release), ("tui/target/release", dev)] {
        if let Some(body) = body {
            executable(&sandbox.root.join("herdr").join(dir).join("flightdeck"), body);
        }
    }
    let out = sandbox.sh("app.sh", &["demo", "flightdeck-space"], &[("FLIGHTDECK_APP_WAIT", "0")]);
    (out.status.code(), stdout(&out), stderr(&out))
}

const NO_FETCH: &str = "#!/bin/sh\necho fetched\nexit 0\n";

#[test]
fn app_the_release_binary_comes_before_a_dev_build() {
    assert_eq!(run_app(Some("#!/bin/sh\necho release \"$@\"\n"), Some("#!/bin/sh\necho dev\n"), NO_FETCH).1, "release demo flightdeck-space\n");
    assert_eq!(run_app(None, Some("#!/bin/sh\necho dev \"$@\"\n"), NO_FETCH).1, "dev demo flightdeck-space\n");
}

#[test]
fn app_the_binary_knows_its_herdr_folder() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("app.sh");
    executable(&sandbox.root.join("herdr/bin/flightdeck"), "#!/bin/sh\necho \"$FLIGHTDECK_HERDR_DIR\"\n");
    let out = sandbox.sh("app.sh", &["demo"], &[("FLIGHTDECK_HERDR_DIR", "/elsewhere")]);
    assert_eq!(PathBuf::from(stdout(&out).trim()), sandbox.root.join("herdr"));
}

#[test]
fn app_missing_fetches_the_release_then_runs_it() {
    let fetch = "#!/bin/sh\nmkdir -p bin && printf '#!/bin/sh\\necho fetched \"$@\"\\n' > bin/flightdeck && chmod +x bin/flightdeck\n";
    assert_eq!(run_app(None, None, fetch).1, "fetched demo flightdeck-space\n");
}

#[test]
fn app_missing_everywhere_says_so_and_fails() {
    let (code, out, err) = run_app(None, None, NO_FETCH);
    assert_eq!((code, out.as_str()), (Some(1), "fetched\n"));
    assert_eq!(err, "flightdeck: no flightdeck binary for this machine yet; run flightdeck doctor to see why\n");
}

// ---------------------------------------------------------------------------
// flightdeck.sh

#[test]
fn flightdeck_sh_runs_the_release_binary_else_the_dev_build() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("flightdeck.sh");
    executable(&sandbox.root.join("herdr/tui/target/release/flightdeck"), "#!/bin/sh\necho dev \"$@\"\n");
    assert_eq!(stdout(&sandbox.sh("flightdeck.sh", &["actions", "sidebar"], &[])), "dev actions sidebar\n");
    executable(&sandbox.root.join("herdr/bin/flightdeck"), "#!/bin/sh\necho release \"$@\" \"$FLIGHTDECK_HERDR_DIR\"\n");
    let printed = stdout(&sandbox.sh("flightdeck.sh", &["hub", "ping"], &[]));
    assert_eq!(printed, format!("release hub ping {}\n", sandbox.root.join("herdr").display()));
}

#[test]
fn flightdeck_sh_without_a_binary_exits_quietly() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("flightdeck.sh");
    let out = sandbox.sh("flightdeck.sh", &["actions", "sidebar"], &[]);
    assert_eq!((out.status.code(), stdout(&out).as_str(), stderr(&out).as_str()), (Some(0), "", ""));
}

// ---------------------------------------------------------------------------
// browse.sh and shots.sh

/// The sandbox's herdr folder with these scripts and a flightdeck that prints its arguments.
fn with_flightdeck(names: &[&str]) -> Sandbox {
    let sandbox = Sandbox::new();
    for name in names {
        sandbox.copy_script(name);
    }
    executable(&sandbox.root.join("herdr/bin/flightdeck"), &common::fake("flightdeck", &sandbox.log(), &[("*", "    echo ran")]));
    sandbox
}

#[test]
fn browse_and_shots_hand_their_arguments_over() {
    let sandbox = with_flightdeck(&["browse.sh", "shots.sh"]);
    let out = sandbox.sh("browse.sh", &["--size", "0.4", "https://x.test/a b", "s-1"], &[]);
    assert_eq!((out.status.code(), stdout(&out).as_str()), (Some(0), "ran\n"));
    sandbox.sh("shots.sh", &["https://x.test", "/out", "--sizes", "mobile", "--full"], &[]);
    assert_eq!(sandbox.calls(), ["flightdeck browse --size 0.4 https://x.test/a b s-1", "flightdeck shots https://x.test /out --sizes mobile --full"]);
}

#[test]
fn browse_and_shots_without_the_binary_say_so() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("browse.sh");
    sandbox.copy_script("shots.sh");
    let out = sandbox.sh("browse.sh", &["https://x.test"], &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("no flightdeck binary"));
    let out = sandbox.sh("shots.sh", &["https://x.test", "/out"], &[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("take these screenshots in Chrome instead"));
}

// ---------------------------------------------------------------------------
// palette.sh

#[test]
fn palette_opens_its_popup_told_of_the_pane_below() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("palette.sh");
    sandbox.herdr(&[("pane current", r#"{"result": {"pane": {"pane_id": "w2:p1", "agent": "claude"}}}"#)]);
    assert_eq!(sandbox.sh("palette.sh", &[], &[]).status.code(), Some(0));
    let open = "herdr plugin pane open --plugin flightdeck --entrypoint palette --placement popup --env PALETTE_PANE=";
    assert_eq!(sandbox.calls(), ["herdr pane current".to_string(), format!("{open}w2:p1")]);
    std::fs::write(sandbox.log(), "").unwrap();
    sandbox.sh("palette.sh", &[], &[("HERDR_PANE_ID", "w5:p3")]);
    assert_eq!(sandbox.calls(), [format!("{open}w5:p3")]);
}

// ---------------------------------------------------------------------------
// whats-new.sh

#[test]
fn whats_new_opens_its_popup_only_while_an_update_waits_to_be_shown() {
    for (pending, opened) in [(0, true), (1, false)] {
        let sandbox = Sandbox::new();
        for name in ["whats-new.sh", "lib.sh", "flightdeck.sh"] {
            sandbox.copy_script(name);
        }
        executable(
            &sandbox.root.join("herdr/bin/flightdeck"),
            &common::fake("flightdeck", &sandbox.log(), &[("options pending", &format!("    exit {pending}"))]),
        );
        sandbox.herdr(&[]);
        assert_eq!(sandbox.sh("whats-new.sh", &[], &[]).status.code(), Some(0));
        let mut expected = vec!["flightdeck options pending".to_string()];
        if opened {
            expected.push("herdr plugin pane open --plugin flightdeck --entrypoint whats-new --placement popup".into());
        }
        assert_eq!(sandbox.calls(), expected);
    }
}

// ---------------------------------------------------------------------------
// update.sh

/// The pattern update.sh hands `pkill -f` on the line naming what it stops.
fn killed(what: &str) -> regex::Regex {
    let script = std::fs::read_to_string(scripts().join("update.sh")).unwrap();
    let line = script.lines().find(|l| l.starts_with("pkill -f") && l.contains(what)).unwrap_or_else(|| panic!("update.sh kills {what} by pattern"));
    regex::Regex::new(line.trim_start_matches("pkill -f ").trim_matches('"')).unwrap()
}

#[test]
fn update_stops_every_space_app_and_nothing_else() {
    let apps = killed("flightdeck-space");
    for app in flightdeck_core::SPACE_APPS {
        assert!(apps.is_match(&format!("/x/herdr/bin/flightdeck {app} flightdeck-space")), "{app}");
    }
    for line in ["/x/herdr/tui/target/release/flightdeck meetings flightdeck-space", "flightdeck todo flightdeck-space"] {
        assert!(apps.is_match(line), "{line}");
    }
    for line in
        ["/x/herdr/bin/flightdeck show demo", "/x/herdr/bin/flightdeck hub", "/x/herdr/bin/flightdeck actions sidebar", "/x/herdr/bin/flightdeck space inbox"]
    {
        assert!(!apps.is_match(line), "{line}");
    }
}

#[test]
fn update_stops_every_spaces_supervisor_and_nothing_else() {
    let supervisors = killed("flightdeck space");
    for app in flightdeck_core::SPACE_APPS {
        assert!(supervisors.is_match(&format!("/x/herdr/bin/flightdeck space {app}")), "{app}");
    }
    for line in ["/x/herdr/tui/target/release/flightdeck space meetings", "flightdeck space todo", "/x/flightdeck space runbook"] {
        assert!(supervisors.is_match(line), "{line}");
    }
    for line in [
        "/x/herdr/bin/flightdeck inbox flightdeck-space",
        "/x/herdr/bin/flightdeck space inbox extra",
        "/x/herdr/bin/flightdeck hub",
        "/x/notflightdeck space inbox",
    ] {
        assert!(!supervisors.is_match(line), "{line}");
    }
}

#[test]
fn update_stops_the_hub_and_nothing_else() {
    let hub = killed("hub");
    for line in ["/x/herdr/bin/flightdeck hub", "/x/herdr/tui/target/release/flightdeck hub", "flightdeck hub"] {
        assert!(hub.is_match(line), "{line}");
    }
    for line in
        ["/x/herdr/bin/flightdeck hub ping", "/x/herdr/bin/flightdeck inbox flightdeck-space", "/x/herdr/bin/flightdeck show hub", "/x/notflightdeck hub"]
    {
        assert!(!hub.is_match(line), "{line}");
    }
}

// ---------------------------------------------------------------------------
// pin-spaces.sh

#[test]
fn pin_spaces_moves_the_pinned_ones_on_top_in_order() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("pin-spaces.sh");
    sandbox.copy_script("lib.sh");
    sandbox.write("home/.config/flightdeck/config.json", r#"{"spaces": ["inbox", "terminal", {"name": "actions"}, "runbook"]}"#);
    let listed = r#"{"result": {"workspaces": [{"workspace_id": "a", "label": "[1] eng-4"}, {"workspace_id": "b", "label": "[2] x  runbook"}, {"workspace_id": "c", "label": "[3] y  inbox"}, {"workspace_id": "d", "label": "[4] z  actions"}]}}"#;
    sandbox.herdr(&[("workspace list", listed)]);
    let nc =
        format!("#!/bin/sh\n{{ printf 'nc %s ' \"$*\"; cat; }} >> '{}'\necho '{{\"id\": \"flightdeck:pin\", \"result\": {{}}}}'\n", sandbox.log().display());
    executable(&sandbox.root.join("bin/nc"), &nc);
    let out = sandbox.sh("pin-spaces.sh", &[], &[("HERDR_SOCKET_PATH", "/s/herdr.sock")]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        sandbox.calls(),
        [
            "herdr workspace list",
            r#"nc -w 2 -U /s/herdr.sock {"id":"flightdeck:pin","method":"workspace.move","params":{"workspace_id":"c","insert_index":0}}"#,
            r#"nc -w 2 -U /s/herdr.sock {"id":"flightdeck:pin","method":"workspace.move","params":{"workspace_id":"d","insert_index":2}}"#,
        ]
    );
    std::fs::write(sandbox.log(), "").unwrap();
    let ordered = r#"{"result": {"workspaces": [{"workspace_id": "c", "label": "[1] y  inbox"}, {"workspace_id": "a", "label": "[2] eng-4"}]}}"#;
    sandbox.herdr(&[("workspace list", ordered)]);
    sandbox.sh("pin-spaces.sh", &[], &[("HERDR_SOCKET_PATH", "/s/herdr.sock")]);
    assert_eq!(sandbox.calls(), ["herdr workspace list"]);
}

// ---------------------------------------------------------------------------
// ensure-spaces.sh

/// The `flightdeck` on PATH, which an update leaves on a folder it then prunes.
#[test]
fn ensure_spaces_puts_the_flightdeck_on_path_back_on_the_link() {
    let sandbox = Sandbox::new();
    sandbox.copy_script("ensure-spaces.sh");
    sandbox.copy_script("lib.sh");
    executable(&sandbox.root.join("herdr/fetch-apps.sh"), "#!/bin/sh\nexit 0\n");
    executable(&sandbox.root.join("herdr/bin/flightdeck"), "#!/bin/sh\necho ran\n");
    sandbox.write("home/.config/flightdeck/config.json", r#"{"spaces": []}"#);
    let link = sandbox.home().join(".config/flightdeck/herdr");
    std::os::unix::fs::symlink(sandbox.root.join("herdr"), &link).unwrap();
    let bin = sandbox.home().join(".local/bin/flightdeck");
    std::fs::create_dir_all(bin.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(sandbox.root.join("gone/bin/flightdeck"), &bin).unwrap();
    let out = sandbox.sh("ensure-spaces.sh", &[], &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(std::fs::read_link(&bin).unwrap(), link.join("bin/flightdeck"));
}

// ---------------------------------------------------------------------------
// fetch-apps.sh

/// The release target of this machine, as fetch-apps.sh names it.
fn target() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => return None,
    })
}

/// A fake GitHub release: files served from a folder over HTTP, each requested path kept.
struct Release {
    dir: tempfile::TempDir,
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl Release {
    fn serve() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let (root, seen) = (dir.path().to_path_buf(), requests.clone());
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut line = String::new();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                // The headers, up to the blank line.
                let mut header = String::new();
                while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
                    header.clear();
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                seen.lock().unwrap().push(path.clone());
                let (status, body) = match std::fs::read(root.join(path.trim_start_matches('/'))) {
                    Ok(body) => ("200 OK", body),
                    Err(_) => ("404 Not Found", b"Not Found".to_vec()),
                };
                let head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(head.as_bytes()).and_then(|_| stream.write_all(&body));
            }
        });
        Release { dir, url, requests }
    }

    fn asset(&self) -> PathBuf {
        self.dir.path().join("v9.9.9").join(format!("flightdeck-apps-9.9.9-{}.tar.gz", target().unwrap()))
    }

    /// Publishes an archive of the binary, a script saying "released", not executable, with its checksum file (a wrong
    /// one when given).
    fn publish(&self, checksum: Option<&str>) {
        let staging = tempfile::tempdir().unwrap();
        std::fs::write(staging.path().join("flightdeck"), "#!/bin/sh\necho released \"$@\"\n").unwrap();
        let asset = self.asset();
        std::fs::create_dir_all(asset.parent().unwrap()).unwrap();
        let packed = Command::new("tar").env("COPYFILE_DISABLE", "1").arg("-C").arg(staging.path()).arg("-czf").arg(&asset).arg("flightdeck").status().unwrap();
        assert!(packed.success());
        let digest = match checksum {
            Some(sum) => sum.to_string(),
            None => sha256(&asset),
        };
        let name = asset.file_name().unwrap().to_string_lossy().into_owned();
        std::fs::write(asset.with_file_name(format!("{name}.sha256")), format!("{digest}  {name}\n")).unwrap();
    }
}

fn sha256(path: &Path) -> String {
    let out = Command::new("shasum").args(["-a", "256"]).arg(path).output().or_else(|_| Command::new("sha256sum").arg(path).output()).unwrap();
    String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap().to_string()
}

/// A herdr folder with fetch-apps.sh, on version 9.9.9.
fn plugin() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.copy_script("fetch-apps.sh");
    sandbox.write("herdr/herdr-plugin.toml", "id = \"flightdeck\"\nversion = \"9.9.9\"\n");
    sandbox
}

fn fetch(sandbox: &Sandbox, release: &Release) -> std::process::Output {
    sandbox.sh("fetch-apps.sh", &[], &[("FLIGHTDECK_RELEASES_URL", &release.url)])
}

fn listing(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

fn can_fetch() -> bool {
    target().is_some() && Command::new("curl").arg("--version").output().is_ok_and(|o| o.status.success())
}

#[test]
fn fetch_installs_the_binary_once_per_version() {
    if !can_fetch() {
        return;
    }
    let (sandbox, release) = (plugin(), Release::serve());
    release.publish(None);
    let out = fetch(&sandbox, &release);
    assert_eq!((out.status.code(), stdout(&out).as_str(), stderr(&out).as_str()), (Some(0), "", ""));
    let bin = sandbox.root.join("herdr/bin");
    assert_eq!(stdout(&Command::new(bin.join("flightdeck")).output().unwrap()), "released\n");
    assert_eq!(std::fs::read_to_string(bin.join(".version")).unwrap(), "9.9.9\n");
    let seen = release.requests.lock().unwrap().len();
    assert_eq!(fetch(&sandbox, &release).status.code(), Some(0));
    assert_eq!(release.requests.lock().unwrap().len(), seen);
    assert_eq!(listing(&sandbox.root.join("herdr")), ["bin", "fetch-apps.sh", "herdr-plugin.toml"]);
}

#[test]
fn fetch_from_a_release_without_the_apps_changes_nothing() {
    if !can_fetch() {
        return;
    }
    let (sandbox, release) = (plugin(), Release::serve());
    let out = fetch(&sandbox, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    assert!(!sandbox.root.join("herdr/bin").exists());
}

#[test]
fn fetch_with_a_wrong_checksum_installs_nothing_and_says_so() {
    if !can_fetch() {
        return;
    }
    let (sandbox, release) = (plugin(), Release::serve());
    release.publish(Some(&"0".repeat(64)));
    let out = fetch(&sandbox, &release);
    assert_ne!(out.status.code(), Some(0));
    assert!(stderr(&out).contains("checksum mismatch"), "{}", stderr(&out));
    assert_eq!(listing(&sandbox.root.join("herdr")), ["fetch-apps.sh", "herdr-plugin.toml"]);
}

#[test]
fn fetch_in_a_checkout_with_its_own_build_does_nothing() {
    if !can_fetch() {
        return;
    }
    let (sandbox, release) = (plugin(), Release::serve());
    release.publish(None);
    executable(&sandbox.root.join("herdr/tui/target/release/flightdeck"), "");
    assert_eq!(fetch(&sandbox, &release).status.code(), Some(0));
    assert!(release.requests.lock().unwrap().is_empty());
}

/// A flightdeck binary that logs how it was called, under `name`, and exits with `code`.
fn logging_flightdeck(sandbox: &Sandbox, path: &Path, name: &str, code: i32) {
    executable(path, &format!("#!/bin/sh\nprintf '%s\\n' \"{name} $*\" >> '{}'\necho 'its usage' >&2\nexit {code}\n", sandbox.log().display()));
}

#[test]
fn fetch_has_the_folders_binary_do_it() {
    let (sandbox, release) = (plugin(), Release::serve());
    release.publish(None);
    let here = sandbox.root.join("herdr");
    logging_flightdeck(&sandbox, &here.join("bin/flightdeck"), "own", 0);
    let out = fetch(&sandbox, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    assert_eq!(sandbox.calls(), [format!("own update-apps {}", here.display())]);
    assert!(release.requests.lock().unwrap().is_empty());
}

#[test]
fn fetch_says_why_the_binary_could_not_install() {
    let (sandbox, release) = (plugin(), Release::serve());
    logging_flightdeck(&sandbox, &sandbox.root.join("herdr/bin/flightdeck"), "own", 3);
    let out = fetch(&sandbox, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(1), "its usage\n"));
    assert!(release.requests.lock().unwrap().is_empty());
}

#[test]
fn fetch_downloads_itself_when_no_binary_knows_update_apps() {
    if !can_fetch() {
        return;
    }
    let (sandbox, release) = (plugin(), Release::serve());
    release.publish(None);
    logging_flightdeck(&sandbox, &sandbox.root.join("herdr/bin/flightdeck"), "own", 1);
    let out = fetch(&sandbox, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    assert_eq!(stdout(&Command::new(sandbox.root.join("herdr/bin/flightdeck")).output().unwrap()), "released\n");
}

/// herdr's folders for the plugin: `installs/<n>/checkout/herdr` for each name, the last with fetch-apps.sh on 9.9.9.
fn installs(sandbox: &Sandbox, names: &[&str]) -> Vec<PathBuf> {
    let folders: Vec<PathBuf> = names.iter().map(|n| sandbox.root.join("plugins/github-installations/flightdeck").join(n).join("checkout/herdr")).collect();
    let new = folders.last().unwrap();
    std::fs::create_dir_all(new).unwrap();
    std::fs::copy(scripts().join("fetch-apps.sh"), new.join("fetch-apps.sh")).unwrap();
    std::fs::write(new.join("herdr-plugin.toml"), "id = \"flightdeck\"\nversion = \"9.9.9\"\n").unwrap();
    folders
}

fn fetch_in(sandbox: &Sandbox, folder: &Path, release: &Release) -> std::process::Output {
    Command::new("sh").arg(folder.join("fetch-apps.sh")).env_clear().envs(sandbox.env()).env("FLIGHTDECK_RELEASES_URL", &release.url).output().unwrap()
}

#[test]
fn fetch_in_a_new_install_has_the_newest_earlier_one_do_it() {
    let (sandbox, release) = (Sandbox::new(), Release::serve());
    let folders = installs(&sandbox, &["1-a", "2-b", "3-c"]);
    for (folder, name) in folders.iter().zip(["older", "newest"]) {
        logging_flightdeck(&sandbox, &folder.join("bin/flightdeck"), name, 0);
        std::fs::write(folder.join("bin/.version"), "9.9.8\n").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
    }
    let out = fetch_in(&sandbox, &folders[2], &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    assert_eq!(sandbox.calls(), [format!("newest update-apps {}", folders[2].display())]);
}

#[test]
fn fetch_before_the_release_has_the_binary_copies_the_earlier_install_in() {
    if target().is_none() {
        return;
    }
    let (sandbox, release) = (Sandbox::new(), Release::serve());
    let folders = installs(&sandbox, &["1-a", "2-b"]);
    let (old, new) = (&folders[0], &folders[1]);
    std::fs::create_dir_all(old.join("bin")).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_flightdeck"), old.join("bin/flightdeck")).unwrap();
    std::fs::write(old.join("bin/.version"), "0.0.1\n").unwrap();
    // The link still names the old install, as when the scripts of the new one have not run yet.
    std::os::unix::fs::symlink(old, sandbox.home().join(".config/flightdeck/herdr")).unwrap();
    let out = fetch_in(&sandbox, new, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    let own = env!("CARGO_PKG_VERSION");
    assert_eq!(std::fs::read_to_string(new.join("bin/.version")).unwrap(), format!("{own}\n"));
    let copied = stdout(&Command::new(new.join("bin/flightdeck")).arg("--version").output().unwrap());
    assert_eq!(copied, format!("flightdeck {own}\n"));
    // Once the release has it, the copied binary installs it.
    release.publish(None);
    let out = fetch_in(&sandbox, new, &release);
    assert_eq!((out.status.code(), stderr(&out).as_str()), (Some(0), ""));
    assert_eq!(std::fs::read_to_string(new.join("bin/.version")).unwrap(), "9.9.9\n");
    assert_eq!(stdout(&Command::new(new.join("bin/flightdeck")).output().unwrap()), "released\n");
    assert_eq!(listing(new), ["bin", "fetch-apps.sh", "herdr-plugin.toml"]);
}

// ---------------------------------------------------------------------------
// Hubs started on a demo HOME

/// A hub a script starts on its own HOME (the sandbox, the screenshots) fetches nothing and asks no agent of this
/// machine for its models: every one that turns the feed off turns the model listings off too.
#[test]
fn every_script_that_starts_a_demo_hub_asks_no_agent_for_its_models() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files: Vec<PathBuf> = std::fs::read_dir(repo.join("docs/screenshots")).unwrap().flatten().map(|e| e.path()).collect();
    files.extend(std::fs::read_dir(repo.join("scripts")).unwrap().flatten().map(|e| e.path()));
    let mut checked = 0;
    for file in files.iter().filter(|f| f.extension().is_some_and(|e| e == "py" || e == "sh")) {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        if text.contains("FLIGHTDECK_HUB_FEED") {
            checked += 1;
            assert!(text.contains("FLIGHTDECK_HUB_MODELS"), "{} starts a hub that asks this machine's agents for their models", file.display());
        }
    }
    assert!(checked >= 4, "the sandbox and the screenshot scripts: {checked}");
}

// the GUI sandbox's hc

/// The hc a sandbox's hub runs refuses to type into a terminal, which would run on the real HOME, unless asked to.
#[test]
fn the_sandbox_s_hc_refuses_to_type_a_line_unless_allowed() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let log = root.path().join("hc.log");
    std::fs::create_dir_all(home.join(".local/bin")).unwrap();
    executable(&home.join(".local/bin/hc"), &format!("#!/bin/sh\necho \"$HOME|$HC_SESSION|$*\" >> {}\n", log.display()));
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/sandbox.sh");
    let wrapper = root.path().join("box/hc.sh");
    std::fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    let write = |allow: &str| {
        let out = Command::new("sh")
            .arg(&script)
            .args(["hc-wrapper", &wrapper.to_string_lossy()])
            .env("HOME", &home)
            .env("FLIGHTDECK_SANDBOX", root.path().join("box"))
            .env("SANDBOX_ALLOW_SEND", allow)
            .env_remove("HC_SESSION")
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    };
    let run = |args: &[&str], allow: Option<&str>| {
        let mut command = Command::new(&wrapper);
        command.args(args).env("HOME", "/elsewhere").env_remove("SANDBOX_ALLOW_SEND");
        if let Some(allow) = allow {
            command.env("SANDBOX_ALLOW_SEND", allow);
        }
        command.output().unwrap()
    };
    write("");
    let refused = run(&["send", "--force", "--json", "--", "t1", "terminal-browser config set render.fps 60"], None);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&refused.stderr), "sandbox: typing into a terminal with the real HOME is refused\n");
    assert!(!log.exists(), "nothing reached hc");
    // Anything else goes to hc on the HOME and the session the sandbox was started with: the folder's name by default.
    assert!(run(&["ls"], None).status.success());
    assert_eq!(std::fs::read_to_string(&log).unwrap(), format!("{}|box|ls\n", home.display()));
    // Allowed when it runs, or when it was written.
    assert!(run(&["send", "t1", "ls"], Some("1")).status.success());
    write("1");
    assert!(run(&["send", "t1", "pwd"], None).status.success());
    let lines = std::fs::read_to_string(&log).unwrap();
    assert!(lines.contains("|box|send t1 ls\n") && lines.contains("|box|send t1 pwd\n"), "{lines}");
}
