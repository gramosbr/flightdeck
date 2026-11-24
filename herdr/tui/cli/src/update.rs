//! `flightdeck update-apps [herdr folder]`: installs the release's binary for the plugin's version into the folder's bin/,
//! the one fetch-apps.sh runs. It exits 3 when the install failed, so the script tells that from a binary without it.

use std::path::PathBuf;

use flightdeck_core::files;

pub fn main(args: &[String]) {
    let dir = args.first().map(PathBuf::from).unwrap_or_else(files::herdr_dir);
    if let Err(error) = flightdeck_update::update(&dir, &flightdeck_update::releases()) {
        let version = flightdeck_update::wanted(&dir).unwrap_or_default();
        eprintln!("flightdeck: flightdeck {version} could not be installed ({error}) → sh {}/fetch-apps.sh", dir.display());
        std::process::exit(3);
    }
}
