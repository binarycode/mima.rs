mod env;

use env::Env;

#[test]
fn error() {
    let mut env = Env::new();

    env.add_guest_config("zero");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) apply-snapshot zero root
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Unknown snapshot 'root' for guest 'zero'
    "});
}
