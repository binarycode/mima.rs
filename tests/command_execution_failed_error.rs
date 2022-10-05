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

    env.stub(
        format!("qemu-img snapshot -droot {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock"
            echo "Is another process using the image [{sda_path}]?"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock" >&2
            echo "Is another process using the image [{sda_path}]?" >&2
            exit 1
        "#},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) delete-snapshot zero root
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img snapshot -droot {sda_path}'

        stdout:
        qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock
        Is another process using the image [{sda_path}]?

        stderr:
        qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock
        Is another process using the image [{sda_path}]?

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

    env.stub(
        format!("qemu-img snapshot -droot {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock" >&2
            echo "Is another process using the image [{sda_path}]?" >&2
            exit 1
        "#},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) delete-snapshot zero root
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img snapshot -droot {sda_path}'

        stderr:
        qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock
        Is another process using the image [{sda_path}]?

    "});
}

#[test]
fn error_without_stderr() {
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

    env.stub(
        format!("qemu-img snapshot -droot {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock"
            echo "Is another process using the image [{sda_path}]?"
            exit 1
        "#},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) delete-snapshot zero root
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img snapshot -droot {sda_path}'

        stdout:
        qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock
        Is another process using the image [{sda_path}]?

    "});
}

#[test]
fn error_without_streams() {
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

    env.stub_default("qemu-img", "exit 1");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) delete-snapshot zero root
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img snapshot -droot {sda_path}'

    "});
}
