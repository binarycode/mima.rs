mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn happy_path_with_aliases() {
    let mut env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();
    file.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_ok("ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima");
    env.stub_ok(format!{
        "scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/",
        file_path,
    });

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {
        "
            ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima
            scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/
        ",
        file_path,
    });

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy zero ((file_path))
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {
        "
            ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima
            scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/
        ",
        file_path,
    });

    command_macros::command!(
        {env.bin()} -c (env.config_path()) upload zero ((file_path))
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {
        "
            ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima
            scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/
        ",
        file_path,
    });
}

#[test]
fn failure_when_path_is_not_a_file() {
    let mut env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();

    env.add_guest_config("zero");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {
        "
            Error: `{}` is not a file
        ",
        file_path,
    });
}

#[test]
fn ssh_failure() {
    let mut env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();
    file.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    // TODO: real failure output
    env.stub(
        "ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {r#"
        Error: Failed to run "ssh" "-o" "ConnectionAttempts=3" "-o" "ConnectTimeout=60" "-o" "BatchMode=yes" "-o" "StrictHostKeyChecking=no" "-o" "UserKnownHostsFile=/dev/null" "root@1.1.1.1" "mkdir" "-p" "/root/mima"
        stdout:
        foobar

        stderr:


    "#});

    env.assert_history(indoc::indoc! {"
        ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima
    "});
}

#[test]
fn scp_failure() {
    let mut env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();
    file.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_ok("ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima");
    // TODO: real failure output
    env.stub(
        format!(
            "scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/",
            file_path,
        ),
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {
        r#"
            Error: Failed to run "scp" "-o" "ConnectionAttempts=3" "-o" "ConnectTimeout=60" "-o" "BatchMode=yes" "-o" "StrictHostKeyChecking=no" "-o" "UserKnownHostsFile=/dev/null" "{}" "root@1.1.1.1:/root/mima/"
            stdout:
            foobar

            stderr:


        "#,
        file_path,
    });

    env.assert_history(indoc::formatdoc! {
        "
            ssh -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 mkdir -p /root/mima
            scp -o ConnectionAttempts=3 -o ConnectTimeout=60 -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {} root@1.1.1.1:/root/mima/
        ",
        file_path,
    });
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>
            <PATH>

        USAGE:
            mima copy-file-to-guest <GUEST_ID> <PATH>

        For more information try --help
    "});
}

#[test]
fn one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest one
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <PATH>

        USAGE:
            mima copy-file-to-guest <GUEST_ID> <PATH>

        For more information try --help
    "});
}

#[test]
fn more_than_two_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest one two three
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'three' which wasn't expected, or isn't valid in this context

        USAGE:
            mima copy-file-to-guest <GUEST_ID> <PATH>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();
    file.touch().unwrap();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        Error: Unknown guest `zero`
    "});
}
