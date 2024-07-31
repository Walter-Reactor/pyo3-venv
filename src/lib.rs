use anyhow::Result;
use std::{
    env,
    ffi::OsString,
    iter::once,
    path::Path,
    process::{self},
};
use tempfile::TempDir;

/// Helper trait for running a command you expect to succeed
trait CheckedCommand {
    fn run_checked(&mut self) -> Result<process::Output>;
}
impl CheckedCommand for process::Command {
    fn run_checked(&mut self) -> Result<process::Output> {
        let out = self.output()?;
        if out.status.success() {
            return Ok(out);
        }

        Err(anyhow::format_err!(
            "Error {} while running command: {:?}. Output: \n{}\n{}",
            out.status,
            self,
            String::from_utf8(out.stdout)?,
            String::from_utf8(out.stderr)?
        ))
    }
}

/// Simple type for setting up & running commands within a python venv
pub struct PyVEnv {
    #[allow(dead_code)]
    venv_dir: Option<TempDir>,
    path: OsString,
}

impl PyVEnv {
    fn get_venv_path(&self) -> &Path {
        match &self.venv_dir {
            Some(i) => i.path(),
            None => Path::new(".venv"),
        }
    }

    /// Create a new venv in a uniquely named temp directory which will be removed on drop
    pub fn new() -> Result<Self> {
        let venv_dir = tempfile::tempdir()?;
        let venv_scripts_path = venv_dir.path().join(if env::consts::OS == "windows" { "Scripts" } else { "bin" });
        let path = env::join_paths(once(venv_scripts_path).chain(env::split_paths(&env::var("PATH")?)))?;

        process::Command::new("uv").args(["venv"]).arg(venv_dir.path()).run_checked()?;

        PyVEnv { venv_dir: Some(venv_dir), path }.install(&["pytest", "maturin"])
    }

    /// Create a venv in the local .venv folder. Does *not* overwrite the existing .venv
    pub fn new_persistant() -> Result<Self> {
        let venv_scripts_path = Path::new(".venv").join(if env::consts::OS == "windows" { "Scripts" } else { "bin" });
        let path = env::join_paths(once(venv_scripts_path).chain(env::split_paths(&env::var("PATH")?)))?;

        process::Command::new("uv").args(["venv", "--seed", "--allow-existing"]).run_checked()?;

        PyVEnv { venv_dir: None, path }.install(&["pytest", "maturin"])
    }

    /// Returns a new command with the venv environment configured
    pub fn cmd(&self, cmd: &str) -> process::Command {
        let mut new_command = process::Command::new(cmd);
        new_command.envs([("PATH", self.path.as_os_str()), ("VIRTUAL_ENV", self.get_venv_path().as_os_str())]);
        new_command
    }

    /// Run install with the given args
    pub fn install(self, args: &[&str]) -> Result<Self> {
        self.cmd("uv").args(["pip", "install"]).args(args).run_checked()?;
        Ok(self)
    }

    /// Execute maturin develop in the current directory
    pub fn maturin_develop(self) -> Result<Self> {
        self.add_maturin_dep(Path::new("."))
    }

    /// Attempt to add the given directory to the venv as a maturin package
    pub fn add_maturin_dep(self, path: &Path) -> Result<Self> {
        self.cmd("maturin").current_dir(path).args(["develop", "--uv"]).run_checked().map(|_| self)
    }

    /// Execute a python module
    pub fn run_module(&self, module: &str, args: &[&str]) -> Result<process::Output> {
        self.cmd("python").arg("-m").arg(module).args(args).run_checked()
    }

    /// Run pytest
    pub fn run_pytest(&self) -> Result<()> {
        // Pass -s to ensure that on failure we capture *all* test output
        // Without this, rust panic backtraces are swollowed
        self.cmd("pytest").arg("--version").run_checked()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use crate::PyVEnv;

    #[test]
    fn run_pytest() -> Result<()> {
        PyVEnv::new()?
            .run_pytest()
    }
}
