mod env;

use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) help delete-snapshot
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-delete-snapshot 0.13.0
        Delete snapshot

        USAGE:
            mima delete-snapshot <GUEST_ID> <SNAPSHOT_ID>

        ARGS:
            <GUEST_ID>       Guest ID
            <SNAPSHOT_ID>    Snapshot ID

        OPTIONS:
            -h, --help    Print help information
    "});
}

#[test]
fn happy_path() {
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
        {env.bin()} -c (env.config_path()) delete-snapshot zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {"
        qemu-img snapshot -droot {sda_path}
    "});
}

#[test]
fn multiple_disks() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    let sdb = env.child("zero-sdb.qcow2");
    let sdb_path = sdb.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
                {{ label = 'sdb', path = '{sdb_path}', size = 100 }},
            ]
    "});

    env.stub_default_ok("qemu-img");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {"
        qemu-img snapshot -droot {sda_path}
        qemu-img snapshot -droot {sdb_path}
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>
            <SNAPSHOT_ID>

        USAGE:
            mima delete-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot one
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
            error: The following required arguments were not provided:
                <SNAPSHOT_ID>

            USAGE:
                mima delete-snapshot <GUEST_ID> <SNAPSHOT_ID>

            For more information try --help
    "});
}

#[test]
fn more_than_two_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot one two three
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'three' which wasn't expected, or isn't valid in this context

        USAGE:
            mima delete-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}

#[test]
fn snapshot_removal_failure() {
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

    command_macros::command!(
        {env.bin()} -c (env.config_path()) delete-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {r#"
        error: Failed to run 'qemu-img snapshot -droot {sda_path}'

        stdout:
        qemu-img: Could not open '{sda_path}': Failed to get "write" lock
        Is another process using the image [{sda_path}]?

    "#});

    env.assert_history(indoc::formatdoc! {"
        qemu-img snapshot -droot {sda_path}
    "});
}
