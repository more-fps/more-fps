use crate::Error;
use log::debug;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Requirements<'a> {
    pub args: &'a str,
    pub current_dir: PathBuf,
}

impl<'a> TryFrom<&'a str> for Requirements<'a> {
    type Error = Error;
    fn try_from(args: &'a str) -> Result<Self, Self::Error> {
        let current_dir = env::current_dir()?;
        Ok(Self { args, current_dir })
    }
}

pub fn run(binary: &str, requirements: Requirements, error: Error) -> Result<String, Error> {
    debug!(
        "cd {} && {binary} {};",
        requirements.current_dir.display(),
        requirements.args
    );
    let args = shell_words::split(requirements.args)?;
    let output = Command::new(binary)
        .args(args)
        .current_dir(requirements.current_dir)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        debug!("stdout: {stdout}");
        debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err(error);
    }
    debug!("Finished executing command");
    Ok(stdout.to_string())
}
