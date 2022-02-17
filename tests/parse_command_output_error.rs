mod env;

use env::Env;

#[test]
fn error() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
            ]
    "});

    env.stub_default("qemu-img", "echo 'foobar'");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to parse output of 'qemu-img info --force-share --output=json {sda_path}'

        stdout:
        foobar

    "});
}

#[test]
fn error_without_stdout() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
            ]
    "});

    env.stub_default_ok("qemu-img");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to parse output of 'qemu-img info --force-share --output=json {sda_path}'

    "});
}
