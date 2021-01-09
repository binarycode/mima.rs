mod env;

use env::Env;

#[test]
fn simple_happy_path_with_aliases() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '/tmp/zero.socket'
            pidfile_path = '/tmp/zero.pid'
    "});

    env.stub_ok("qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::indoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start zero
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::indoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});
}

#[test]
fn happy_path_with_complex_configuration() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [networks.pub]
            bridge_name = 'mima-pub'
        [networks.mgt]
            bridge_name = 'mima-mgt'
        [networks.san]
            bridge_name = 'mima-san'
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '/tmp/zero.socket'
            pidfile_path = '/tmp/zero.pid'
            network_interfaces = [
                { network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' },
                { network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero' },
                { network = 'san', mac_address = '52:54:00:00:0A:10', tap_name = 'mima-san0-zero', model = 'e1000e' },
                { network = 'san', mac_address = '52:54:00:01:0A:10', tap_name = 'mima-san1-zero', model = 'e1000e' },
            ]
            disks = [
                { label = 'sda', path = '/tmp/zero.sda.qcow2', size = 20 },
                { label = 'sdb', path = '/tmp/zero.sdb.qcow2', size = 100 },
            ]
    "});

    env.stub_ok("qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san0-zero,mac=52:54:00:00:0A:10 -netdev tap,id=network.mima-san0-zero,ifname=mima-san0-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san1-zero,mac=52:54:00:01:0A:10 -netdev tap,id=network.mima-san1-zero,ifname=mima-san1-zero,script=no,downscript=no -device scsi-hd,drive=drive.sda -drive if=none,id=drive.sda,format=qcow2,file=/tmp/zero.sda.qcow2 -device scsi-hd,drive=drive.sdb -drive if=none,id=drive.sdb,format=qcow2,file=/tmp/zero.sdb.qcow2 -boot d -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file=/tmp/centos7.iso -drive if=floppy,id=drive.fd0,format=raw,file=fat:floppy:rw:/tmp/zero.ks");
    env.stub_ok("ip link set mima-pub-zero master mima-pub up");
    env.stub_ok("ip link set mima-mgt-zero master mima-mgt up");
    env.stub_ok("ip link set mima-san0-zero master mima-san up");
    env.stub_ok("ip link set mima-san1-zero master mima-san up");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero --cdrom /tmp/centos7.iso --floppy /tmp/zero.ks
    )
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::indoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san0-zero,mac=52:54:00:00:0A:10 -netdev tap,id=network.mima-san0-zero,ifname=mima-san0-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san1-zero,mac=52:54:00:01:0A:10 -netdev tap,id=network.mima-san1-zero,ifname=mima-san1-zero,script=no,downscript=no -device scsi-hd,drive=drive.sda -drive if=none,id=drive.sda,format=qcow2,file=/tmp/zero.sda.qcow2 -device scsi-hd,drive=drive.sdb -drive if=none,id=drive.sdb,format=qcow2,file=/tmp/zero.sdb.qcow2 -boot d -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file=/tmp/centos7.iso -drive if=floppy,id=drive.fd0,format=raw,file=fat:floppy:rw:/tmp/zero.ks
        ip link set mima-pub-zero master mima-pub up
        ip link set mima-mgt-zero master mima-mgt up
        ip link set mima-san0-zero master mima-san up
        ip link set mima-san1-zero master mima-san up
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>

        USAGE:
            mima start-guest [OPTIONS] <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn more_than_one_argument() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest one two
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'two' which wasn't expected, or isn't valid in this context

        USAGE:
            mima start-guest [OPTIONS] <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        Error: Unknown guest `zero`
    "});
}

#[test]
fn guest_start_failure() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '/tmp/zero.socket'
            pidfile_path = '/tmp/zero.pid'
    "});

    // TODO: real failure output
    env.stub(
        "qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {r#"
        Error: Failed to run "qemu-system-x86_64" "-name" "zero" "-machine" "q35,accel=kvm" "-cpu" "host" "-m" "8192M" "-smp" "4" "-no-user-config" "-nodefaults" "-daemonize" "-runas" "nobody" "-monitor" "unix:/tmp/zero.socket,server,nowait" "-pidfile" "/tmp/zero.pid" "-vga" "std" "-spice" "port=5901,disable-ticketing=on" "-object" "iothread,id=iothread1" "-device" "virtio-scsi-pci-non-transitional,iothread=iothread1"
        stdout:
        foobar

        stderr:


    "#});

    env.assert_history(indoc::indoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});
}

#[test]
fn iproute_failure() {
    let mut env = Env::new();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [networks.pub]
            bridge_name = 'mima-pub'
        [networks.mgt]
            bridge_name = 'mima-mgt'
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '/tmp/zero.socket'
            pidfile_path = '/tmp/zero.pid'
            network_interfaces = [
                { network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' },
                { network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero' },
            ]
    "});

    env.stub_ok("qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no");
    env.stub_ok("ip link set mima-pub-zero master mima-pub up");
    // TODO: real failure output
    env.stub(
        "ip link set mima-mgt-zero master mima-mgt up",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command!(
        {env.bin()} -c (env.config_path()) start-guest zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {r#"
        Error: Failed to run "ip" "link" "set" "mima-mgt-zero" "master" "mima-mgt" "up"
        stdout:
        foobar

        stderr:


    "#});

    env.assert_history(indoc::indoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:/tmp/zero.socket,server,nowait -pidfile /tmp/zero.pid -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no
        ip link set mima-pub-zero master mima-pub up
        ip link set mima-mgt-zero master mima-mgt up
    "});
}
