mod env;

use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) help apply-snapshot
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-apply-snapshot 0.7.0
        Apply snapshot

        USAGE:
            mima apply-snapshot <GUEST_ID> <SNAPSHOT_ID>

        ARGS:
            <GUEST_ID>       Guest ID
            <SNAPSHOT_ID>    Snapshot ID

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
    env.stub(
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::indoc! {r#"
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
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    let expected_history = indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
        qemu-img snapshot -aroot {sda_path}
    "};

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) restore zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) revert zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);

    command_macros::command!(
        {env.bin()} -c (env.config_path()) switch zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(&expected_history);
}

#[test]
fn common_snapshots_for_multiple_disks() {
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
    env.stub(
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::indoc! {r#"
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
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "centos7",
                            "date-sec": 2,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "1",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "dev",
                            "date-sec": 3,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "2",
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
        "#},
    );
    env.stub(
        format!("qemu-img info --force-share --output=json {sdb_path}"),
        indoc::indoc! {r#"
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
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "centos6",
                            "date-sec": 2,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "1",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "test",
                            "date-sec": 3,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "2",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "dev",
                            "date-sec": 4,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "3",
                            "vm-state-size": 0
                        }
                    ],
                    "virtual-size": 107374182400,
                    "filename": "zero-sdb.qcow2",
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
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .success()
    .stdout("")
    .stderr("");

    env.assert_history(indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
        qemu-img info --force-share --output=json {sdb_path}
        qemu-img snapshot -aroot {sda_path}
        qemu-img snapshot -aroot {sdb_path}
    "});
}

#[test]
fn unknown_snapshot_failure() {
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
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::indoc! {r#"
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
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero dev
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown snapshot 'dev' for guest 'zero'
    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
    "});
}

#[test]
fn uncommon_snapshot_failure() {
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

    env.stub(
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::indoc! {r#"
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
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "centos7",
                            "date-sec": 2,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "1",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "dev",
                            "date-sec": 3,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "2",
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
        "#},
    );
    env.stub(
        format!("qemu-img info --force-share --output=json {sdb_path}"),
        indoc::indoc! {r#"
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
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "centos6",
                            "date-sec": 2,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "1",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "test",
                            "date-sec": 3,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "2",
                            "vm-state-size": 0
                        },
                        {
                            "icount": 0,
                            "vm-clock-nsec": 0,
                            "name": "dev",
                            "date-sec": 4,
                            "date-nsec": 0,
                            "vm-clock-sec": 0,
                            "id": "3",
                            "vm-state-size": 0
                        }
                    ],
                    "virtual-size": 107374182400,
                    "filename": "zero-sdb.qcow2",
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
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero centos7
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown snapshot 'centos7' for guest 'zero'
    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
        qemu-img info --force-share --output=json {sdb_path}
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>
            <SNAPSHOT_ID>

        USAGE:
            mima apply-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot one
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <SNAPSHOT_ID>

        USAGE:
            mima apply-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_two_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot one two three
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'three' which wasn't expected, or isn't valid in this context

        USAGE:
            mima apply-snapshot <GUEST_ID> <SNAPSHOT_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}

#[test]
fn list_snapshots_failure() {
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
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open {sda_path}: Could not open '{sda_path}': No such file or directory"
            exit 1
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-img info --force-share --output=json {sda_path}'

        stdout:
        qemu-img: Could not open {sda_path}: Could not open '{sda_path}': No such file or directory

    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
    "});
}

#[test]
fn apply_snapshot_failure() {
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
        format!("qemu-img info --force-share --output=json {sda_path}"),
        indoc::indoc! {r#"
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
        "#},
    );
    env.stub(
        format!("qemu-img snapshot -aroot {sda_path}"),
        indoc::formatdoc! {r#"
            echo "qemu-img: Could not open '{sda_path}': Failed to get \"write\" lock"
            echo "Is another process using the image [{sda_path}]?"
            exit 1
        "#},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {r#"
        error: Failed to run 'qemu-img snapshot -aroot {sda_path}'

        stdout:
        qemu-img: Could not open '{sda_path}': Failed to get "write" lock
        Is another process using the image [{sda_path}]?

    "#});

    env.assert_history(indoc::formatdoc! {"
        qemu-img info --force-share --output=json {sda_path}
        qemu-img snapshot -aroot {sda_path}
    "});
}
