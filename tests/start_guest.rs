mod env;

use assert_fs::prelude::*;
use env::Env;
use predicates::prelude::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) help start-guest
    }
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-start-guest 0.13.0
        Start guest

        USAGE:
            mima start-guest [OPTIONS] <GUEST_ID>

        ARGS:
            <GUEST_ID>    Guest ID

        OPTIONS:
                --boot-from-cdrom         Boot from CD-ROM
                --boot-from-network       Boot from network
                --cdrom <CDROM_PATH>      Insert CD-ROM image from specified path
                --floppy <FLOPPY_PATH>    Insert floppy image from specified path
            -h, --help                    Print help information
    "});
}

#[test]
fn simple_happy_path_with_aliases() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    let expected_history = indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "};

    env.assert_history(&expected_history);

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(&expected_history);
}

#[test]
fn remote_happy_path() {
    let mut env = Env::new();

    let monitor_socket_parent_dir = env.child("sockets");
    let monitor_socket_parent_dir_path = monitor_socket_parent_dir.path().display();

    let monitor_socket = monitor_socket_parent_dir.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile_parent_dir = env.child("pids");
    let pidfile_parent_dir_path = pidfile_parent_dir.path().display();

    let pidfile = pidfile_parent_dir.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    let sdb = env.child("zero-sdb.qcow2");
    let sdb_path = sdb.path().display();

    let centos7_iso = env.child("centos7.iso");
    let centos7_iso_path = centos7_iso.path().display();

    let kickstart = env.child("zero.ks");
    let kickstart_path = kickstart.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
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
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
            network_interfaces = [
                {{ network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' }},
                {{ network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero' }},
                {{ network = 'san', mac_address = '52:54:00:00:0A:10', tap_name = 'mima-san0-zero', model = 'e1000e' }},
                {{ network = 'san', mac_address = '52:54:00:01:0A:10', tap_name = 'mima-san1-zero', model = 'e1000e' }},
            ]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
                {{ label = 'sdb', path = '{sdb_path}', size = 100 }},
            ]
    "});

    env.stub_default_ok("ssh");
    env.stub(
        format!("ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com pgrep --full --pidfile {pidfile_path} qemu"),
        "exit 1"
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) --host example.com start-guest zero --cdrom ((centos7_iso_path)) --floppy ((kickstart_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which ip
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which pgrep
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which pkill
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which qemu-img
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which qemu-system-x86_64
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com which socat
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com test -e {pidfile_path}
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com test -e {monitor_socket_path}
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com pgrep --full --pidfile {pidfile_path} qemu
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com mkdir --mode 0755 -p {monitor_socket_parent_dir_path}
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com mkdir --mode 0755 -p {pidfile_parent_dir_path}
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san0-zero,mac=52:54:00:00:0A:10 -netdev tap,id=network.mima-san0-zero,ifname=mima-san0-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san1-zero,mac=52:54:00:01:0A:10 -netdev tap,id=network.mima-san1-zero,ifname=mima-san1-zero,script=no,downscript=no -device scsi-hd,drive=drive.sda -drive if=none,id=drive.sda,format=qcow2,file={sda_path} -device scsi-hd,drive=drive.sdb -drive if=none,id=drive.sdb,format=qcow2,file={sdb_path} -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file={centos7_iso_path} -drive if=floppy,id=drive.fd0,format=raw,file=fat:floppy:rw:{kickstart_path}
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com ip link set mima-pub-zero master mima-pub up
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com ip link set mima-mgt-zero master mima-mgt up
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com ip link set mima-san0-zero master mima-san up
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com ip link set mima-san1-zero master mima-san up
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@example.com chmod 644 {pidfile_path}
    "});
}

#[test]
fn setting_pidfile_permissions() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default(
        "qemu-system-x86_64",
        indoc::formatdoc! {"
            touch {pidfile_path}
            chmod 777 {pidfile_path}
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    let permissions = pidfile.metadata().unwrap().permissions().mode();
    assert_eq!(permissions, 0o100644);
}

#[test]
fn happy_path_with_complex_configuration() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    let sda = env.child("zero-sda.qcow2");
    let sda_path = sda.path().display();

    let sdb = env.child("zero-sdb.qcow2");
    let sdb_path = sdb.path().display();

    let centos7_iso = env.child("centos7.iso");
    let centos7_iso_path = centos7_iso.path().display();

    let kickstart = env.child("zero.ks");
    let kickstart_path = kickstart.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
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
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
            network_interfaces = [
                {{ network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' }},
                {{ network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero' }},
                {{ network = 'san', mac_address = '52:54:00:00:0A:10', tap_name = 'mima-san0-zero', model = 'e1000e' }},
                {{ network = 'san', mac_address = '52:54:00:01:0A:10', tap_name = 'mima-san1-zero', model = 'e1000e' }},
            ]
            disks = [
                {{ label = 'sda', path = '{sda_path}', size = 20 }},
                {{ label = 'sdb', path = '{sdb_path}', size = 100 }},
            ]
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));
    env.stub_default_ok("ip");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero --cdrom ((centos7_iso_path)) --floppy ((kickstart_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san0-zero,mac=52:54:00:00:0A:10 -netdev tap,id=network.mima-san0-zero,ifname=mima-san0-zero,script=no,downscript=no -device e1000e,netdev=network.mima-san1-zero,mac=52:54:00:01:0A:10 -netdev tap,id=network.mima-san1-zero,ifname=mima-san1-zero,script=no,downscript=no -device scsi-hd,drive=drive.sda -drive if=none,id=drive.sda,format=qcow2,file={sda_path} -device scsi-hd,drive=drive.sdb -drive if=none,id=drive.sdb,format=qcow2,file={sdb_path} -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file={centos7_iso_path} -drive if=floppy,id=drive.fd0,format=raw,file=fat:floppy:rw:{kickstart_path}
        ip link set mima-pub-zero master mima-pub up
        ip link set mima-mgt-zero master mima-mgt up
        ip link set mima-san0-zero master mima-san up
        ip link set mima-san1-zero master mima-san up
    "});
}

#[test]
fn happy_path_with_boot_from_cdrom() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    let centos7_iso = env.child("centos7.iso");
    let centos7_iso_path = centos7_iso.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero --boot-from-cdrom --cdrom ((centos7_iso_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -boot d -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file={centos7_iso_path}
    "});
}

#[test]
fn happy_path_with_several_cdroms() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    let centos7_iso = env.child("centos7.iso");
    let centos7_iso_path = centos7_iso.path().display();

    let kickstart_iso = env.child("ks.iso");
    let kickstart_iso_path = kickstart_iso.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));
    env.stub_default_ok("ip");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero --cdrom ((centos7_iso_path)) --cdrom ((kickstart_iso_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device scsi-cd,drive=drive.cd0 -drive if=none,id=drive.cd0,format=raw,media=cdrom,file={centos7_iso_path} -device scsi-cd,drive=drive.cd1 -drive if=none,id=drive.cd1,format=raw,media=cdrom,file={kickstart_iso_path}
    "});
}

#[test]
fn happy_path_with_boot_from_network() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero --boot-from-network
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -boot n
    "});
}

#[test]
fn boot_from_more_than_one_source_failure() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero --boot-from-network --boot-from-cdrom
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The argument '--boot-from-network' cannot be used with '--boot-from-cdrom'

        USAGE:
            mima start-guest --boot-from-network <GUEST_ID>

        For more information try --help
    "});
}

#[test]
fn noop_when_guest_is_already_running() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();
    monitor_socket.touch().unwrap();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();
    pidfile.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default_ok("pgrep");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        pgrep --full --pidfile {pidfile_path} qemu
    "});
}

#[test]
fn pidfile_parent_dir_creation() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile_parent_dir = env.child("pids");
    let pidfile = pidfile_parent_dir.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));

    pidfile_parent_dir.assert(predicate::path::missing());

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    pidfile_parent_dir.assert(predicate::path::exists());

    let permissions = pidfile_parent_dir.metadata().unwrap().permissions().mode();
    assert_eq!(permissions, 0o40755);

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});
}

#[test]
fn monitor_socket_parent_dir_creation() {
    let mut env = Env::new();

    let monitor_socket_parent_dir = env.child("sockets");
    let monitor_socket = monitor_socket_parent_dir.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    env.stub_default(
        "qemu-system-x86_64",
        indoc::formatdoc! {"
            touch {pidfile_path}
            touch {monitor_socket_path}
        "},
    );

    monitor_socket_parent_dir.assert(predicate::path::missing());

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    monitor_socket_parent_dir.assert(predicate::path::exists());

    let permissions = monitor_socket_parent_dir
        .metadata()
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(permissions, 0o40755);

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest
    }
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

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest one two
    }
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

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}

#[test]
fn guest_start_failure() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
    "});

    // TODO: real failure output
    env.stub_default(
        "qemu-system-x86_64",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1
    "});
}

#[test]
fn iproute_failure() {
    let mut env = Env::new();

    let monitor_socket = env.child("zero.socket");
    let monitor_socket_path = monitor_socket.path().display();

    let pidfile = env.child("zero.pid");
    let pidfile_path = pidfile.path().display();

    env.add_guest_config("zero");
    env.append_config(indoc::formatdoc! {"
        [networks.pub]
            bridge_name = 'mima-pub'
        [networks.mgt]
            bridge_name = 'mima-mgt'
        [guests.zero]
            memory = 8192
            cores = 4
            spice_port = 5901
            monitor_socket_path = '{monitor_socket_path}'
            pidfile_path = '{pidfile_path}'
            network_interfaces = [
                {{ network = 'pub', mac_address = '52:54:00:00:00:10', tap_name = 'mima-pub-zero' }},
                {{ network = 'mgt', mac_address = '52:54:00:00:09:10', tap_name = 'mima-mgt-zero' }},
            ]
    "});

    env.stub_default("qemu-system-x86_64", format!("touch {pidfile_path}"));
    env.stub_default_ok("ip");
    // TODO: real failure output
    env.stub(
        "ip link set mima-mgt-zero master mima-mgt up",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) start-guest zero
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to run 'ip link set mima-mgt-zero master mima-mgt up'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        qemu-system-x86_64 -name zero -machine q35,accel=kvm -cpu host -m 8192M -smp 4 -no-user-config -nodefaults -daemonize -runas nobody -monitor unix:{monitor_socket_path},server,nowait -pidfile {pidfile_path} -vga std -spice port=5901,disable-ticketing=on -object iothread,id=iothread1 -device virtio-scsi-pci-non-transitional,iothread=iothread1 -device virtio-net-pci-non-transitional,netdev=network.mima-pub-zero,mac=52:54:00:00:00:10 -netdev tap,id=network.mima-pub-zero,ifname=mima-pub-zero,script=no,downscript=no -device virtio-net-pci-non-transitional,netdev=network.mima-mgt-zero,mac=52:54:00:00:09:10 -netdev tap,id=network.mima-mgt-zero,ifname=mima-mgt-zero,script=no,downscript=no
        ip link set mima-pub-zero master mima-pub up
        ip link set mima-mgt-zero master mima-mgt up
    "});
}
