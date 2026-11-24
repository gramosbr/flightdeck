//! `flightdeck usage`: every agent's limits with their reset, source and freshness, its account as it says, and the last
//! 7 days' tokens and estimated spend, from the hub's `usage` view (counted here when no hub answers). On a terminal
//! it is a popup that keeps itself current; otherwise it prints the same page as text.

use std::io::IsTerminal;

use flightdeck_core::term;
use flightdeck_core::usage::{self, Popup};

const USAGE: &str = "flightdeck usage [--json]   every agent's limits and the last 7 days' tokens: a popup on a terminal, else text";

pub fn main(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("--json") => println!("{}", usage::fetch(true)),
        None if std::io::stdout().is_terminal() => {
            let mut popup = Popup::new(usage::fetch(false));
            if let Err(e) = term::run(&mut popup) {
                crate::proc::fail(&format!("flightdeck usage: {e}"));
            }
        }
        None => {
            for (_, line) in usage::text(&usage::fetch(true)) {
                println!("{line}");
            }
        }
        _ => crate::proc::fail(USAGE),
    }
}
