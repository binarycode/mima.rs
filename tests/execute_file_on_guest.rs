mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn help() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) help execute-file-on-guest
    }
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima-execute-file-on-guest 0.13.0
        Execute file on guest

        USAGE:
            mima execute-file-on-guest [OPTIONS] <GUEST_ID> <PATH> [-- <ARGS>...]

        ARGS:
            <GUEST_ID>    Guest ID
            <PATH>        File path
            <ARGS>...     Arguments to pass to the file

        OPTIONS:
                --timeout <MAX_CONNECTION_TIMEOUT>    Maximum SSH connection timeout [default: 100]
            -h, --help                                Print help information
    "});
}

#[test]
fn happy_path_with_aliases() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("scp");
    env.stub_default_ok("ssh");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    let expected_history = indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script
    "};

    env.assert_history(&expected_history);

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-script-on-guest zero ((script_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(&expected_history);

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute zero ((script_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(&expected_history);

    command_macros::command! {
        {env.bin()} --config (env.config_path()) run zero ((script_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(&expected_history);
}

#[test]
fn happy_path_with_extra_arguments() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("scp");
    env.stub_default_ok("ssh");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path)) -- foo bar
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script foo bar
    "});
}

#[test]
fn connection_is_established_from_second_attempt() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("scp");
    env.stub_default_ok("ssh");
    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0",
        "exit 1",
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .success()
    .stderr("")
    .stdout("");

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
        ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script
        ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script
    "});
}

#[test]
fn remote_failure() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();

    env.add_guest_config("zero");

    env.stub_default_ok("ssh");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) --host example.com execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: argument '--host' is not allowed for this command
    "});
}

#[test]
fn failure_to_establish_connection() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0",
        "exit 1",
    );
    // TODO: real failure output
    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest --timeout 3 zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to run 'ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0'

        stdout:
        foobar

    "});

    env.assert_history(indoc::indoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=2 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
    "});
}

#[test]
fn failure_when_path_is_not_a_file() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();

    env.add_guest_config("zero");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: '{script_path}' is not a file
    "});
}

#[test]
fn first_ssh_failure() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("ssh");
    // TODO: real failure output
    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to run 'ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima'

        stdout:
        foobar

    "});

    env.assert_history(indoc::indoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
    "});
}

#[test]
fn scp_failure() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("ssh");
    // TODO: real failure output
    env.stub(
        format!("scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima"),
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to run 'scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
    "});
}

#[test]
fn second_ssh_failure() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("scp");
    env.stub_default_ok("ssh");
    // TODO: real failure output
    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to run 'ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script
    "});
}

#[test]
fn third_ssh_failure() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");
    env.append_config(indoc::indoc! {"
        [guests.zero]
            ip_address = '1.1.1.1'
    "});

    env.stub_default_ok("scp");
    env.stub_default_ok("ssh");
    // TODO: real failure output
    env.stub(
        "ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script",
        indoc::indoc! {"
            echo 'foobar'
            exit 1
        "},
    );

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Failed to run 'ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script'

        stdout:
        foobar

    "});

    env.assert_history(indoc::formatdoc! {"
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null root@1.1.1.1 exit 0
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 mkdir -p /root/mima
        scp -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null {script_path} root@1.1.1.1:/root/mima
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 chmod +x /root/mima/script
        ssh -o BatchMode=yes -o ConnectTimeout=1 -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -A root@1.1.1.1 /root/mima/script
    "});
}

#[test]
fn no_arguments() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <GUEST_ID>
            <PATH>

        USAGE:
            mima execute-file-on-guest [OPTIONS] <GUEST_ID> <PATH> [-- <ARGS>...]

        For more information try --help
    "});
}

#[test]
fn one_argument() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest one
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: The following required arguments were not provided:
            <PATH>

        USAGE:
            mima execute-file-on-guest <GUEST_ID> <PATH>

        For more information try --help
    "});
}

#[test]
fn more_than_two_arguments() {
    let env = Env::new();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest one two three
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Found argument 'three' which wasn't expected, or isn't valid in this context

        USAGE:
            mima execute-file-on-guest [OPTIONS] <GUEST_ID> <PATH> [-- <ARGS>...]

        For more information try --help
    "});
}

#[test]
fn unknown_guest() {
    let env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-file-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Unknown guest 'zero'
    "});
}
