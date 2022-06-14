mod env;

use env::Env;

#[test]
fn error() {
    let env = Env::new();

    env.create_binary("which", "exit 1");

    command_macros::command!(
        {env.bin()} -c (env.config_path()) list-guests
    )
    .assert()
    .failure()
    .stdout("")
    .stderr(indoc::indoc! {"
        error: Dependency missing: 'ip', 'pgrep', 'pkill', 'qemu-img', 'qemu-system-x86_64', 'socat'
    "});
}
