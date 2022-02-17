mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn error() {
    let env = Env::new();

    let config = env.child("invalid_config.toml");
    let config_path = config.path().display();
    config.write_str("invalid").unwrap();

    command_macros::command!(
        {env.bin()} -c ((config_path)) list-guests
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to parse configuration in '{config_path}'
    "});
}
