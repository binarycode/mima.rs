mod env;

use env::Env;

#[test]
fn happy_path() {
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
        format!("qemu-img info --force-share --output=json {}", sda_path),
        indoc::indoc! {
            r#"
                echo '
                    {
                        "snapshots": [
                            {
                                "icount": 0,
                                "vm-clock-nsec": 0,
                                "name": "root",
                                "date-sec": 1,
                                "date-nsec": 0,
                                "vm-clock-sec": 0,
                                "id": "0",
                                "vm-state-size": 0
                            }
                        ],
                        "virtual-size": 21474836480,
                        "filename": "zero-sda.qcow2",
                        "cluster-size": 65536,
                        "format": "qcow2",
                        "actual-size": 0,
                        "format-specific": {
                            "type": "qcow2",
                            "data": {
                                "compat": "1.1",
                                "compression-type": "zlib",
                                "lazy-refcounts": false,
                                "refcount-bits": 16,
                                "corrupt": false,
                                "extended-l2": false
                            }
                        },
                        "dirty-flag": false
                    }
                '
            "#,
        },
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img info --force-share --output=json {}
        ",
        sda_path,
    });
}

#[test]
fn happy_negative_path() {
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
        format!("qemu-img info --force-share --output=json {}", sda_path),
        indoc::indoc! {
            r#"
                echo '
                    {
                        "snapshots": [
                            {
                                "icount": 0,
                                "vm-clock-nsec": 0,
                                "name": "root",
                                "date-sec": 1,
                                "date-nsec": 0,
                                "vm-clock-sec": 0,
                                "id": "0",
                                "vm-state-size": 0
                            }
                        ],
                        "virtual-size": 21474836480,
                        "filename": "zero-sda.qcow2",
                        "cluster-size": 65536,
                        "format": "qcow2",
                        "actual-size": 0,
                        "format-specific": {
                            "type": "qcow2",
                            "data": {
                                "compat": "1.1",
                                "compression-type": "zlib",
                                "lazy-refcounts": false,
                                "refcount-bits": 16,
                                "corrupt": false,
                                "extended-l2": false
                            }
                        },
                        "dirty-flag": false
                    }
                '
            "#,
        },
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot zero centos7
    )
    .assert()
    .failure()
    .code(1)
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img info --force-share --output=json {}
        ",
        sda_path,
    });
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>
            <SNAPSHOT_ID>

        USAGE:
            mima check-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot one
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <SNAPSHOT_ID>

        USAGE:
            mima check-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_two_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot one two three
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'three' which wasn't expected, or isn't valid in this context

        USAGE:
            mima check-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        Error: Unknown guest `zero`
    "});
}

#[test]
fn list_snapshots_failure() {
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
        format!("qemu-img info --force-share --output=json {}", sda_path),
        indoc::formatdoc! {
            r#"
                echo "qemu-img: Could not open {0}: Could not open '{0}': No such file or directory"
                exit 1
            "#,
            sda_path,
        },
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) check-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {
        r#"
            Error: Failed to run "qemu-img" "info" "--force-share" "--output=json" "{0}"
            stdout:
            qemu-img: Could not open {0}: Could not open '{0}': No such file or directory

            stderr:


        "#,
        sda_path,
    });

    env.assert_history(indoc::formatdoc! {
        "
            qemu-img info --force-share --output=json {}
        ",
        sda_path,
    });
}