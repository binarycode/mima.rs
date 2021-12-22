mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn happy_path_with_aliases() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {
        "
            [guests.zero]
                disks = [
                    {{ label = 'sda', path = '{}', size = 20 }},
                ]
        ",
        sda_path,
    });

    env.stub_ok(format!(
        "qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 20G",
        sda_path
    ));
    env.stub_ok(format!("qemu-img snapshot -croot {}", sda_path));

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    let expected_history = indoc::formatdoc! {
        "
            qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {0} 20G
            qemu-img snapshot -croot {0}
        ",
        sda_path,
    };

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
    env.append_config(indoc::formatdoc! {
        "
            [guests.zero]
                disks = [
                    {{ label = 'sda', path = '{sda_path}', size = 20 }},
                    {{ label = 'sdb', path = '{sdb_path}', size = 100 }},
                ]
        ",
        sda_path = sda_path,
        sdb_path = sdb_path,
    });

    env.stub_ok(format!(
        "qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 20G",
        sda_path,
    ));
    env.stub_ok(format!(
        "qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 100G",
        sdb_path,
    ));
    env.stub_ok(format!("qemu-img snapshot -croot {}", sda_path));
    env.stub_ok(format!("qemu-img snapshot -croot {}", sdb_path));

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sda_path} 20G
            qemu-img snapshot -croot {sda_path}
            qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {sdb_path} 100G
            qemu-img snapshot -croot {sdb_path}
        ",
        sda_path = sda_path,
        sdb_path = sdb_path,
    });
}

#[test]
fn noop_when_path_exists() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();
    sda.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {
        "
            [guests.zero]
                disks = [
                    {{ label = 'sda', path = '{}', size = 20 }},
                ]
        ",
        sda_path,
    });

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
        Error: Unknown guest `zero`
    "});
}

#[test]
fn disk_creation_failure() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {
        "
            [guests.zero]
                disks = [
                    {{ label = 'sda', path = '{}', size = 20 }},
                ]
        ",
        sda_path,
    });

    env.stub(
        format!(
            "qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 20G",
            sda_path,
        ),
        indoc::formatdoc! {
            r#"
                echo "qemu-img: {0}: Could not create '{0}': No such file or directory"
                exit 1
            "#,
            sda_path,
        },
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {
        r#"
            Error: Failed to run "qemu-img" "create" "-q" "-fqcow2" "-olazy_refcounts=on" "-opreallocation=metadata" "{0}" "20G"
            stdout:
            qemu-img: {0}: Could not create '{0}': No such file or directory

            stderr:


        "#,
        sda_path,
    });

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 20G
        ",
        sda_path,
    });
}

#[test]
fn snapshot_creation_failure() {
    let mut env = Env::new();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {
        "
            [guests.zero]
                disks = [
                    {{ label = 'sda', path = '{}', size = 20 }},
                ]
        ",
        sda_path,
    });

    env.stub_ok(format!(
        "qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {} 20G",
        sda_path
    ));
    env.stub(
        format!("qemu-img snapshot -croot {}", sda_path),
        indoc::formatdoc! {
            r#"
                echo "qemu-img: Could not open '{0}': Failed to get \"write\" lock"
                echo "Is another process using the image [{0}]?"
                exit 1
            "#,
            sda_path,
        },
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) initialize-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {
        r#"
            Error: Failed to run "qemu-img" "snapshot" "-croot" "{0}"
            stdout:
            qemu-img: Could not open '{0}': Failed to get "write" lock
            Is another process using the image [{0}]?

            stderr:


        "#,
        sda_path,
    });

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata {0} 20G
            qemu-img snapshot -croot {0}
        ",
        sda_path,
    });
}
