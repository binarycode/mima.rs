mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) help stop-guest
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-stop-guest 0.8.1
        Stop guest

        USAGE:
            mima stop-guest [OPTIONS] <GUEST_ID>

        ARGS:
            <GUEST_ID>    Guest ID

        OPTIONS:
                --wait <WAIT>    Seconds to wait for soft shutdown [default: 60]
                --force          Kill the guest immediately
            -h, --help           Print help information
    "});
}

#[test]
fn simple_happy_path_with_aliases() {
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
        {env.bin()} -c (env.config_path()) stop-guest zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");
}

#[test]
fn happy_path_when_the_guest_is_not_running() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("pgrep", "exit 1");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn happy_path_with_force_flag() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default_ok("pgrep");
    env.stub_default_ok("pkill");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest --force zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
        pkill --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn happy_path_with_soft_shutdown() {
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
        format!("[ -e {flag_path} ] && exit 1"),
    );
    env.stub(
        format!("socat - UNIX-CONNECT:{monitor_socket_path}"),
        format!("touch {flag_path}"),
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
        socat - UNIX-CONNECT:{monitor_socket_path}
        pgrep --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn happy_path_with_soft_shutdown_timeout() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default_ok("pgrep");
    env.stub_default_ok("pkill");
    env.stub_default_ok("socat");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest --wait 1 zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
        socat - UNIX-CONNECT:{monitor_socket_path}
        pgrep --full --pidfile {pidfile_path} qemu
        pkill --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>

        USAGE:
            mima stop-guest [OPTIONS] <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest one two
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'two' which wasn't expected, or isn't valid in this context

        USAGE:
            mima stop-guest [OPTIONS] <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}

#[test]
fn pkill_failure() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default_ok("pgrep");
    // TODO: real failure output
    env.stub(
        format!("pkill --full --pidfile {pidfile_path} qemu"),
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) stop-guest --force zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'pkill --full --pidfile {pidfile_path} qemu'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
        pkill --full --pidfile {pidfile_path} qemu
    "});
}
