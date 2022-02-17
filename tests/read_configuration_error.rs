mod env;

use env::Env;

#[test]
fn error() {
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
        error: Failed to read configuration from '{config_path}'
    "});
}
