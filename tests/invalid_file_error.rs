mod env;

use env::Env;

#[test]
fn error() {
    let env = Env::new();

    let file = env.child("file");
    let file_path = file.path().display();

    command_macros::command!(
        {env.bin()} -c (env.config_path()) copy-file-to-guest zero ((file_path))
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::formatdoc! {"
        error: '{file_path}' is not a file
    "});
}
