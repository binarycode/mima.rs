mod env;

use env::Env;

#[test]
fn error() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            network_interfaces = [
                { network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' },
            ]
    "});

    env.stub_default_ok("qemu-system-x86_64");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Unknown network 'pub'
    "});
}
