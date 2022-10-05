mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn error() {
    let mut env = Env::new();

    let script = env.child("script");
    let script_path = script.path().display();
    script.touch().unwrap();

    env.add_guest_config("zero");

    command_macros::command! {
        {env.bin()} --config (env.config_path()) execute-script-on-guest zero ((script_path))
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: IP address is not configured for guest 'zero'
    "});
}
