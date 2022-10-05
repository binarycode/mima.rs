mod env;

use assert_fs::prelude::*;
use env::Env;

#[test]
fn error() {
    let env = Env::new();

    let dir = env.child("dir");
    let dir_path = dir.path().display();

    let child = env.child("dir/foo");
    child.touch().unwrap();

    command_macros::command! {
        {env.bin()} --config ((dir_path)) list-guests
    }
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: Failed to read configuration from '{dir_path}'
    "});
}
