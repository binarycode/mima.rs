mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn happy_path_with_aliases() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {
        "
            [networks.pub]
                bridge_name = 'mima-pub'
            [networks.mgt]
                bridge_name = 'mima-mgt'
            [guests.zero]
                memory = 8192
                cores = 4
                description = 'Test Virtual Machine'
                spice_port = 5901
                monitor_socket_path = '{monitor_socket_path}'
                pidfile_path = '{pidfile_path}'
                network_interfaces = [
                    {{ network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' }},
                    {{ network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero', model = 'e1000e' }},
                ]
                disks = [
                    {{ label = 'sda', path = '/mnt/mima/zero/sda.qcow2', size = 20 }},
                    {{ label = 'sdb', path = '/mnt/mima/zero/sdb.qcow2', size = 100 }},
                ]
        ",
        monitor_socket_path = monitor_socket_path,
        pidfile_path = pidfile_path,
    });

    env.stub_ok(format!("pgrep --full --pidfile {} qemu", pidfile_path));

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show-guest-details zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        GUEST  ID    BOOTED  SPICE  MEMORY  CORES  DESCRIPTION
               zero  true    5901   8192    4      Test Virtual Machine

        DISKS  LABEL  SIZE  PATH
               sda    20    /mnt/mima/zero/sda.qcow2
               sdb    100   /mnt/mima/zero/sdb.qcow2

        NETWORK INTERFACES  NETWORK  MODEL                            MAC                TAP
                            pub      virtio-net-pci-non-transitional  52:54:00:00:00:10  mima-pub-zero
                            mgt      e1000e                           52:54:00:00:09:10  mima-mgt-zero
    "});

    env.assert_history(indoc::formatdoc! {
        "
            pgrep --full --pidfile {} qemu
        ",
        pidfile_path,
    });

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        GUEST  ID    BOOTED  SPICE  MEMORY  CORES  DESCRIPTION
               zero  true    5901   8192    4      Test Virtual Machine

        DISKS  LABEL  SIZE  PATH
               sda    20    /mnt/mima/zero/sda.qcow2
               sdb    100   /mnt/mima/zero/sdb.qcow2

        NETWORK INTERFACES  NETWORK  MODEL                            MAC                TAP
                            pub      virtio-net-pci-non-transitional  52:54:00:00:00:10  mima-pub-zero
                            mgt      e1000e                           52:54:00:00:09:10  mima-mgt-zero
    "});

    env.assert_history(indoc::formatdoc! {
        "
            pgrep --full --pidfile {} qemu
        ",
        pidfile_path,
    });

    command_macros::command!(
        {env.bin()} -c (env.config_path()) guest zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        GUEST  ID    BOOTED  SPICE  MEMORY  CORES  DESCRIPTION
               zero  true    5901   8192    4      Test Virtual Machine

        DISKS  LABEL  SIZE  PATH
               sda    20    /mnt/mima/zero/sda.qcow2
               sdb    100   /mnt/mima/zero/sdb.qcow2

        NETWORK INTERFACES  NETWORK  MODEL                            MAC                TAP
                            pub      virtio-net-pci-non-transitional  52:54:00:00:00:10  mima-pub-zero
                            mgt      e1000e                           52:54:00:00:09:10  mima-mgt-zero
    "});

    env.assert_history(indoc::formatdoc! {
        "
            pgrep --full --pidfile {} qemu
        ",
        pidfile_path,
    });
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show-guest-details
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
            error: The following required arguments were not provided:
                <GUEST_ID>

            USAGE:
                mima show-guest-details <GUEST_ID>

            For more information try --help
        "});
}

#[test]
fn more_than_one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show-guest-details one two
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
            error: Found argument 'two' which wasn't expected, or isn't valid in this context

            USAGE:
                mima show-guest-details <GUEST_ID>

            For more information try --help
        "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show-guest-details zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        Error: Unknown guest `zero`
    "});
}