mod env;

use env::Env;

#[test]
fn error_when_config_path_is_specified() {
    let env = Env::new();

    let config = env.child("missing_config.toml");
    let config_path = config.path().display();

    command_macros::command!(
        {env.bin()} -c ((config_path)) list-guests
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Missing configuration at '{config_path}'
    "});
}

#[test]
fn error_when_config_path_is_not_specified() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} list-guests
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Missing configuration at './mima.toml', '/etc/mima.toml'
    "});
}
