use std::fs::{create_dir_all, write};

use anyhow::Error;
use which::which;

mod acceptance_helpers;
use acceptance_helpers::{assert_cmd, set_executable, setup};

#[test]
fn test_basic() -> Result<(), Error> {
    let harness = setup()?;
    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;

    assert_cmd!(harness, fastenv "reload",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands (1 new). Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);
    harness.which("hello").unwrap_err();
    assert_cmd!(harness, fastenv "shim" "hello",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    harness.which("hello")?;
    assert_cmd!(harness, hello,  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    assert_cmd!(harness, fastenv "unshim" "hello",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Removed 1 shims from [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv shim <command>' to add them again
    "###);
    which("hello").unwrap_err();

    assert_cmd!(harness, fastenv "reload",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands. Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);
    Ok(())
}

#[test]
fn test_shadowed() -> Result<(), Error> {
    let mut harness = setup()?;
    harness.prepend_path(harness.join("bogus"));
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;
    assert_cmd!(harness, hello,  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    assert_cmd!(harness, fastenv "shim" "hello",  @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] [scrubbed $HOME]/.fastenv/bin/hello is shadowed by an executable of the same name at [scrubbed $HOME]/project/bogus/hello
    "###);
    Ok(())
}

#[test]
fn test_shadowing() -> Result<(), Error> {
    let harness = setup()?;
    assert_cmd!(harness, fastenv "shim" "true",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    Ok(())
}

#[test]
fn test_shim_self() -> Result<(), Error> {
    let harness = setup()?;
    assert_cmd!(harness, fastenv "unshim" "fastenv",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] not unshimming own binary
    Removed 0 shims from [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv shim <command>' to add them again
    "###);
    assert_cmd!(harness, fastenv "shim" "fastenv",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] not shimming own binary
    created no new shims.
    "###);
    assert_cmd!(harness, fastenv "unshim" "fastenv",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] not unshimming own binary
    Removed 0 shims from [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv shim <command>' to add them again
    "###);
    Ok(())
}

#[test]
fn test_verbosity() -> Result<(), Error> {
    let mut harness = setup()?;
    assert_cmd!(harness, fastenv "vars",  @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] failed to find .envrc in current or any parent directory
    "###);
    harness.set_var("QUICKENV_LOG", "debug");
    assert_cmd!(harness, fastenv "vars",  @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [DEBUG fastenv] argv[0] is "[scrubbed $HOME]/.fastenv/fastenv_bin/fastenv"
    [DEBUG fastenv] own program name is fastenv, so no shim running
    [ERROR fastenv] failed to find .envrc in current or any parent directory
    "###);
    Ok(())
}

#[test]
fn test_script_failure() -> Result<(), Error> {
    let harness = setup()?;
    write(harness.join(".envrc"), "exit 1")?;
    assert_cmd!(harness, fastenv "reload",  @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] .envrc exited with status exit status: 1
    "###);
    Ok(())
}

#[test]
fn test_eating_own_tail() -> Result<(), Error> {
    let harness = setup()?;
    assert_cmd!(harness, fastenv "shim" "bash",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    write(
        harness.join(".envrc"),
        "bash -c 'echo hello world'; export PATH=bogus:$PATH",
    )?;
    assert_cmd!(harness, fastenv "reload",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;
    assert_cmd!(harness, fastenv "shim" "hello",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    assert_cmd!(harness, hello,  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    Ok(())
}

#[test]
fn test_eating_own_tail2() -> Result<(), Error> {
    let harness = setup()?;
    assert_cmd!(harness, fastenv "shim" "bash",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    write(
        harness.join(".envrc"),
        "echo the value is $MYVALUE\nexport MYVALUE=canary",
    )?;
    assert_cmd!(harness, fastenv "reload",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    the value is

    ----- stderr -----
    "###);
    // assert that during reloading, we're not shimming bash and accidentally sourcing the old
    // envvar values. canary should not appear during reload.
    assert_cmd!(harness, fastenv "reload",  @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    the value is

    ----- stderr -----
    "###);
    Ok(())
}

#[test]
fn test_exec() -> Result<(), Error> {
    let harness = setup()?;

    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;

    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands (1 new). Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);

    harness.which("hello").unwrap_err();
    assert_cmd!(harness, fastenv "exec" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    Ok(())
}

#[test]
fn test_shim_creating_shims() -> Result<(), Error> {
    let harness = setup()?;

    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);

    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;

    // there is a command. it does not create more commands. fastenv should not amend any output
    assert_cmd!(harness, fastenv "exec" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);

    // shimming the command should work
    assert_cmd!(harness, fastenv "shim" "--yes", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Found these unshimmed commands in your .envrc:

    hello

    Quickenv will create this new shim binary in [scrubbed $HOME]/.fastenv/bin/.
    Inside of [scrubbed $HOME]/project, those commands will run with .envrc enabled.
    Outside, they will run normally.
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    Use 'fastenv shim <command>' to run additional commands with .envrc enabled.
    "###);

    // change the command such that it creates another command, and run it
    write(
        harness.join("bogus/hello"),
        "#!/bin/sh\necho 'echo hello world' > bogus/hello2 && chmod +x bogus/hello2",
    )?;
    set_executable(harness.join("bogus/hello"))?;

    // fastenv should warn that more commands need shimming now
    assert_cmd!(harness, fastenv "exec" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands (1 new). Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);

    // fastenv shim should find the new command
    assert_cmd!(harness, fastenv "shim" "--yes", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Found these unshimmed commands in your .envrc:

    hello2

    Quickenv will create this new shim binary in [scrubbed $HOME]/.fastenv/bin/.
    Inside of [scrubbed $HOME]/project, those commands will run with .envrc enabled.
    Outside, they will run normally.
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    Use 'fastenv shim <command>' to run additional commands with .envrc enabled.
    "###);

    Ok(())
}

#[test]
fn test_auto_shimming() -> Result<(), Error> {
    let harness = setup()?;

    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;

    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands (1 new). Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);

    assert_cmd!(harness, fastenv "shim" "-y", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Found these unshimmed commands in your .envrc:

    hello

    Quickenv will create this new shim binary in [scrubbed $HOME]/.fastenv/bin/.
    Inside of [scrubbed $HOME]/project, those commands will run with .envrc enabled.
    Outside, they will run normally.
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    Use 'fastenv shim <command>' to run additional commands with .envrc enabled.
    "###);

    assert_cmd!(harness, fastenv "shim" "-y", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    created no new shims.
    Use 'fastenv shim <command>' to run additional commands with .envrc enabled.
    "###);

    Ok(())
}

#[test]
fn test_no_envrc_context() -> Result<(), Error> {
    let harness = setup()?;
    assert_cmd!(harness, fastenv "shim" "echo", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    assert_cmd!(harness, echo "hello world", @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world

    ----- stderr -----
    "###);
    Ok(())
}

#[test]
fn test_eating_own_tail3() -> Result<(), Error> {
    // regression: we removed our own PATH from the PATH envvar, but:
    // 1) the path is actually duplicated, so we didn't remove all copies, and we recurse into
    //    the shim
    // 2) the shim re-adds the PATH through its envrc cache
    let mut harness = setup()?;
    harness.prepend_path(std::fs::canonicalize(harness.join("../.fastenv/bin")).unwrap());
    write(harness.join(".envrc"), "export PATH=hello:$PATH:")?;
    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);
    assert_cmd!(harness, fastenv "shim" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    harness.set_var("QUICKENV_LOG", "debug");
    assert_cmd!(harness, hello, @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [DEBUG fastenv] argv[0] is "[scrubbed $HOME]/.fastenv/bin/hello"
    [DEBUG fastenv] attempting to launch shim for "[scrubbed $HOME]/.fastenv/bin/hello"
    [DEBUG fastenv] loading [scrubbed $HOME]/project/.envrc
    [DEBUG fastenv] removing own entry from PATH: [scrubbed $HOME]/.fastenv/bin
    [DEBUG fastenv] removing own entry from PATH: [scrubbed $HOME]/.fastenv/bin
    [ERROR fastenv] failed to run shimmed command

    Caused by:
        0: failed to run hello
        1: failed to find actual binary
        2: failed to find hello
        3: cannot find binary path
    "###);
    Ok(())
}

#[test]
fn test_which() -> Result<(), Error> {
    let harness = setup()?;

    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    create_dir_all(harness.join("bogus"))?;
    write(harness.join("bogus/hello"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/hello"))?;

    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    [WARN fastenv] 1 unshimmed commands (1 new). Use 'fastenv shim' to make them available.
    Set QUICKENV_NO_SHIM_WARNINGS=1 to silence this message.
    "###);
    assert_cmd!(harness, fastenv "which" "hello", @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] cannot find binary path
    "###);
    assert_cmd!(harness, fastenv "shim" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Created 1 new shims in [scrubbed $HOME]/.fastenv/bin/.
    Use 'fastenv unshim <command>' to remove them again.
    "###);
    assert_cmd!(harness, fastenv "which" "hello", @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    bogus/hello

    ----- stderr -----
    "###);
    Ok(())
}

#[test]
fn test_which_pretend_shimmed() -> Result<(), Error> {
    let harness = setup()?;

    write(harness.join(".envrc"), "export PATH=bogus:$PATH\n")?;
    create_dir_all(harness.join("bogus"))?;

    assert_cmd!(harness, fastenv "which" "bash", @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] "bash" is not shimmed by fastenv
    "###);

    assert_cmd!(harness, fastenv "reload", @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    "###);

    assert_cmd!(harness, fastenv "which" "bash", @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] "bash" is not shimmed by fastenv
    "###);

    write(harness.join("bogus/bash"), "#!/bin/sh\necho hello world")?;
    set_executable(harness.join("bogus/bash"))?;

    assert_cmd!(harness, fastenv "which" "bash", @r###"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    [ERROR fastenv] "bash" is not shimmed by fastenv
    "###);

    assert_cmd!(harness, fastenv "which" "bash" "--pretend-shimmed", @r###"
    success: true
    exit_code: 0
    ----- stdout -----
    bogus/bash

    ----- stderr -----
    "###);
    Ok(())
}
