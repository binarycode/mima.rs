mod env;

use env::Env;

#[test]
fn happy_path() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} help
    )
    .assert()
    .success()
    .stderr("")
    .stdout(indoc::indoc! {"
        mima 0.6.0

        Igor Sidorov <igor.sidorov@binarycode.ru>

        USAGE:
            mima [OPTIONS] <SUBCOMMAND>

        FLAGS:
            -h, --help    Print help information

        OPTIONS:
            -c, --config <CONFIG_PATH>    Path to configuration [default: ./mima.toml]

        SUBCOMMANDS:
            list-guests                   List all guests
            show-guest-details            Show guest details
            initialize-guest              Initialize guest
            start-guest                   Start guest
            stop-guest                    Stop guest
            wait-for-guest-to-shutdown    Wait until the guest shuts down
            copy-file-to-guest            Copy file to guest
            execute-script-on-guest       Execute script on guest
            list-snapshots                List snapshots
            create-snapshot               Create new snapshot
            delete-snapshot               Delete snapshot
            apply-snapshot                Apply snapshot
            check-snapshot                Check if snapshot exists
            help                          Print this message or the help of the given subcommand(s)
    "});
}
