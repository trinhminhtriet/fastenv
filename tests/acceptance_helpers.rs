use std::collections::BTreeMap;
use std::env::{current_dir, var};
use std::ffi::{OsStr, OsString};
use std::fs::{create_dir_all, set_permissions, Permissions};
use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::Error;
use tempfile::TempDir;

pub struct Harness {
    pub env: BTreeMap<OsString, OsString>,
    pub home: Option<TempDir>,
    pub cwd: PathBuf,
}

impl Harness {
    pub fn insta_settings(&self) -> insta::Settings {
        let mut insta_settings = insta::Settings::clone_current();
        insta_settings.add_filter(
            &regex::escape(self.var("HOME").unwrap().to_str().unwrap()),
            "[scrubbed $$HOME]",
        );
        insta_settings.add_filter(
            &regex::escape(
                which::which("true")
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            "[scrubbed usr-bin]",
        );
        insta_settings.add_filter(
            &regex::escape(
                which::which("bash")
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            ),
            "[scrubbed usr-bin2]",
        );
        insta_settings
    }

    pub fn prepend_path(&mut self, path: impl AsRef<OsStr>) {
        let mut new_path = path.as_ref().to_owned();
        new_path.push(":");
        new_path.push(self.var("PATH").unwrap());

        self.set_var("PATH", new_path);
    }

    pub fn var(&self, key: impl AsRef<OsStr>) -> Option<&OsStr> {
        self.env.get(key.as_ref()).map(OsString::as_os_str)
    }

    pub fn set_var(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        self.env
            .insert(key.as_ref().to_owned(), value.as_ref().to_owned());
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.cwd.join(path)
    }

    pub fn which(&self, binary_name: impl AsRef<OsStr>) -> which::Result<PathBuf> {
        which::which_in(binary_name, self.var("PATH"), &self.cwd)
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        if std::env::var("QUICKENV_TEST_LEAK_FILES").unwrap_or_default() == "1" {
            std::mem::forget(self.home.take());
        }
    }
}

pub fn setup() -> Result<Harness, Error> {
    let home = tempfile::tempdir()?;
    // on macos, /tmp is a symlink to /private/..., so sometimes the path reported by tmpdir is not
    // canonical
    let home_path = std::fs::canonicalize(home.path()).unwrap();
    let cwd = home_path.join("project/");
    create_dir_all(&cwd)?;
    let mut harness = Harness {
        env: BTreeMap::new(),
        home: Some(home),
        cwd,
    };

    create_dir_all(home_path.join(".fastenv/bin"))?;
    create_dir_all(home_path.join(".fastenv/fastenv_bin"))?;
    symlink(
        current_dir()?.join("target/debug/fastenv"),
        home_path.join(".fastenv/fastenv_bin/fastenv"),
    )?;

    harness.set_var("HOME", &home_path);
    harness.set_var("PATH", var("PATH").unwrap());
    harness.prepend_path(home_path.join(".fastenv/bin"));
    harness.prepend_path(home_path.join(".fastenv/fastenv_bin"));
    Ok(harness)
}

pub fn set_executable(path: impl AsRef<Path>) -> Result<(), Error> {
    set_permissions(path, Permissions::from_mode(0o755))?;
    Ok(())
}

#[allow(unused_macros)]
macro_rules! assert_cmd {
    ($harness:expr, $program_name:ident $($arg:literal)*, $($insta_args:tt)*) => {{
        use std::process::Command;

        let _guard = $harness.insta_settings().bind_to_scope();
        insta_cmd::assert_cmd_snapshot!(
            Command::new($harness.which(stringify!($program_name))?)
            .current_dir(&$harness.cwd)
            .env_remove("QUICKENV_PRELUDE")
            .envs(&$harness.env)
            $(.arg($arg))*,
            $($insta_args)*
        );
    }}
}

#[allow(unused_imports)]
pub(crate) use assert_cmd;
