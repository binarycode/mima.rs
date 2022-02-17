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
        {env.bin()} -c (env.config_path()) create-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Disk 'sda' of guest 'zero' already contains snapshot 'root'
    "});
}
