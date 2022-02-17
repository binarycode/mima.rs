mod env;

use assert_fs::prelude::*;
use env::Env;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

#[test]
fn error() {
    let env = Env::new();

    let which = env.child("which");
    which
        .write_str(&indoc::indoc! {"
            #! /usr/bin/env bash
            exit 1
        "})
        .unwrap();
    let permissions = Permissions::from_mode(0o777);
    std::fs::set_permissions(&which.path(), permissions).unwrap();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) list-guests
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Dependency missing: 'ip', 'pgrep', 'pkill', 'qemu-img', 'qemu-system-x86_64', 'socat'
    "});
}
