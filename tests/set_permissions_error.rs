mod env;

use env::Env;

#[test]
fn error() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '/tmp/zero.socket'
            pidfile_path = '/'
    "});

    env.stub_default_ok("qemu-system-x86_64");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to set permissions '644' on '/'
    "});
}
