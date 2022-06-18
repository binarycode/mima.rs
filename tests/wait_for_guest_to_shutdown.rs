mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) help "wait-for-guest-to-shutdown"
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-wait-for-guest-to-shutdown 0.10.0
        Wait until the guest shuts down

        USAGE:
            mima wait-for-guest-to-shutdown <GUEST_ID>

        ARGS:
            <GUEST_ID>    Guest ID

        OPTIONS:
            -h, --help    Print help information
    "});
}

#[test]
fn simple_happy_path_with_alias() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    command_macros::command!(
        {env.bin()} -c (env.config_path()) "wait-for-guest-to-shutdown" zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) wait zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");
}

#[test]
fn happy_path_with_wait() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    let flag = env.child("flag");
    let flag_path = flag.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub(
        format!("pgrep --full --pidfile {pidfile_path} qemu"),
        indoc::formatdoc! {"
            [ -e {flag_path} ] && exit 1
            touch {flag_path}
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) "wait-for-guest-to-shutdown" zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
        pgrep --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) "wait-for-guest-to-shutdown"
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>

        USAGE:
            mima wait-for-guest-to-shutdown <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) "wait-for-guest-to-shutdown" one two
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'two' which wasn't expected, or isn't valid in this context

        USAGE:
            mima wait-for-guest-to-shutdown <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) "wait-for-guest-to-shutdown" zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}
