mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) help initialize-guest
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-initialize-guest 0.9.1
        Initialize guest

        USAGE:
            mima initialize-guest <GUEST_ID>

        ARGS:
            <GUEST_ID>    Guest ID

        OPTIONS:
            -h, --help    Print help information
    "});
}

#[test]
fn happy_path_with_aliases() {
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
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    let expected_history = indoc::formatdoc! {"
        qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G
        qemu-img snapshot -croot {sda_path}
    "};

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) init zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) init-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);
}

#[test]
fn happy_path_with_multiple_disks() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    let sdb = env.child("zerb-sda.qcow2");
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
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {"
        qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G
        qemu-img snapshot -croot {sda_path}
        qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sdb_path} 100G
        qemu-img snapshot -croot {sdb_path}
    "});
}

#[test]
fn noop_when_path_exists() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();
    sda.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
            ]
    "});

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history("");
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>

        USAGE:
            mima initialize-guest <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest one two
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'two' which wasn't expected, or isn't valid in this context

        USAGE:
            mima initialize-guest <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}

#[test]
fn disk_creation_failure() {
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
        format!("qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G"),
        indoc::formatdoc! {r#"
            echo "qemu-img: {sda_path}: Could not create '{sda_path}': No such file or directory"
            exit 1
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G'

        stdout:
        qemu-img: {sda_path}: Could not create '{sda_path}': No such file or directory

    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G
    "});
}

#[test]
fn snapshot_creation_failure() {
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
    env.stub(
        format!("qemu-img snapshot -croot {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock"
            echo "Is another process using the image [{sda_path}]?"
            exit 1
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {r#"
        error: Failed to run 'qemu-img snapshot -croot {sda_path}'

        stdout:
        qemu-img: Could not open '{sda_path}': Failed to get "write" lock
        Is another process using the image [{sda_path}]?

    "#});

    env.assert_history(indoc::formatdoc! {"
        qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G
        qemu-img snapshot -croot {sda_path}
    "});
}
