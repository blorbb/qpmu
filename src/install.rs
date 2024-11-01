use std::process::ExitStatus;

use color_eyre::eyre::Result;

use crate::PLUGINS_DIR;

pub fn install_plugin(args: &[String]) -> Result<ExitStatus> {
    // TODO: don't use unstable feature
    Ok(std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-wasip2")
        .arg("--artifact-dir")
        .arg(&*PLUGINS_DIR)
        .arg("-Z")
        .arg("unstable-options")
        .args(args)
        .spawn()?
        .wait()?)
}
