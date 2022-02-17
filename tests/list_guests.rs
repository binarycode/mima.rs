mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn happy_path_with_aliases() {
    let mut env = Env::new();

    let beta_monitor_socket = env.child("beta.socket");
    let beta_monitor_socket_path = beta_monitor_socket.path().display();
    beta_monitor_socket.touch().unwrap();

    let zero_monitor_socket = env.child("zero.socket");
    let zero_monitor_socket_path = zero_monitor_socket.path().display();
    zero_monitor_socket.touch().unwrap();

    let beta_pidfile = env.child("beta.pid");
    let beta_pidfile_path = beta_pidfile.path().display();
    beta_pidfile.touch().unwrap();

    let zero_pidfile = env.child("zero.pid");
    let zero_pidfile_path = zero_pidfile.path().display();
    zero_pidfile.touch().unwrap();

    env.add_guest_config("beta");
    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.beta]
            description = 'foo'
            spice_port = 5901
            monitor_socket_path = '{beta_monitor_socket_path}'
            pidfile_path = '{beta_pidfile_path}'
        [guests.zero]
            description = 'bar'
            spice_port = 5902
            monitor_socket_path = '{zero_monitor_socket_path}'
            pidfile_path = '{zero_pidfile_path}'
    "});

    env.stub_default_ok("pgrep");
    env.stub(
        format!("pgrep --full --pidfile {beta_pidfile_path} qemu"),
        "exit 1",
    );

    let expected_output = indoc::indoc! {"
        ID    BOOTED  SPICE  DESCRIPTION
        beta  false   5901   foo
        zero  true    5902   bar
    "};

    command_macros::command!(
        {env.bin()} -c (env.config_path()) list-guests
    )
    .assert()
    .success()
    .stderr("")
    .stdout(expected_output);

    let expected_history = indoc::formatdoc! {"
        pgrep --full --pidfile {beta_pidfile_path} qemu
        pgrep --full --pidfile {zero_pidfile_path} qemu
    "};

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) list
    )
    .assert()
    .success()
    .stderr("")
    .stdout(expected_output);

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) guests
    )
    .assert()
    .success()
    .stderr("")
    .stdout(expected_output);

    env.assert_history(&expected_history);
}

#[test]
fn more_than_zero_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) list-guests one
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'one' which wasn't expected, or isn't valid in this context

        USAGE:
            mima list-guests

        For more information try --help
    "});
}
