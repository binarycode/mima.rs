mod env;

use env::Env;

#[test]
fn error() {
    let env = Env::new();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) show-guest-details zero
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Unknown guest 'zero'
    "});
}
