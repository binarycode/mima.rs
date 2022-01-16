use assert_cmd::Command;
use assert_fs::assert::IntoPathPredicate;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use merge_yaml_hash::MergeYamlHash;
use predicates_core::Predicate;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub struct Env {
    bin_path_env: String,
    config_file: ChildPath,
    flows: HashMap<String, BTreeMap<String, String>>,
    history_file: ChildPath,
    tmp_dir: TempDir,
}

#[allow(dead_code)]
impl Env {
    pub fn new() -> Self {
        let tmp_dir = TempDir::new().unwrap();

        let bin_path_env = vec![
            command_path_str("bash"),
            command_path_str("cat"),
            tmp_dir.path().display().to_string(),
        ]
        .join(":");

        let config_file = tmp_dir.child("mima.toml");
        config_file
            .write_str(indoc::indoc! {"
                [networks]
                [guests]
            "})
            .unwrap();

        let flows = HashMap::new();

        let history_file = tmp_dir.child("history");
        history_file.touch().unwrap();

        let which = tmp_dir.child("which");
        which
            .write_str(&indoc::indoc! {"
                #! /usr/bin/env bash
                exit 0
            "})
            .unwrap();
        let permissions = Permissions::from_mode(0o777);
        std::fs::set_permissions(&which.path(), permissions).unwrap();

        Self {
            bin_path_env,
            config_file,
            flows,
            history_file,
            tmp_dir,
        }
    }

    pub fn bin(&self) -> Command {
        let mut bin = Command::cargo_bin("mima").unwrap();
        bin.env("PATH", &self.bin_path_env);
        bin
    }

    pub fn config_path(&self) -> &Path {
        self.config_file.path()
    }

    pub fn child<T>(&self, path: T) -> ChildPath
    where
        T: AsRef<Path>,
    {
        self.tmp_dir.child(path)
    }

    pub fn add_guest_config<T>(&mut self, guest_id: T)
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        self.append_config(indoc::formatdoc! {"
            [guests.{guest_id}]
                memory = 4096
                cores = 2
                description = '{guest_id}'
                monitor_socket_path = '/tmp/{guest_id}.socket'
                pidfile_path = '/tmp/{guest_id}.pid'
                spice_port = 5900
                network_interfaces = []
                disks = []
        "});
    }

    pub fn append_config<T>(&mut self, config: T)
    where
        T: AsRef<str>,
    {
        let config = config.as_ref();

        let config_toml = std::fs::read_to_string(&self.config_file).unwrap();

        let config_yaml = toml::from_str::<serde_yaml::Value>(&config_toml).unwrap();
        let config_yaml = serde_yaml::to_string(&config_yaml).unwrap();

        let yaml = toml::from_str::<serde_yaml::Value>(config).unwrap();
        let yaml = serde_yaml::to_string(&yaml).unwrap();

        let mut hash = MergeYamlHash::new();
        hash.merge(&config_yaml);
        hash.merge(&yaml);
        let hash = hash;

        let config_yaml = hash.to_string();

        let config_toml = serde_yaml::from_str::<toml::Value>(&config_yaml).unwrap();
        let config_toml = toml::to_string(&config_toml).unwrap();

        self.config_file.write_str(&config_toml).unwrap();
    }

    pub fn stub<T, U>(&mut self, command: T, script: U)
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let command = command.as_ref();
        let script = script.as_ref();

        let mut splitter = command.splitn(2, " ");
        let bin = splitter.next().unwrap();
        let arguments = splitter.next().unwrap();

        let bin_flows = self.flows.entry(bin.to_owned()).or_default();
        bin_flows.insert(arguments.to_owned(), script.to_owned());

        let mut scripts = String::new();
        for (arguments, script) in bin_flows {
            scripts.push_str(&indoc::formatdoc! {
                r#"
                    if [[ "$@" == "{arguments}" ]]; then
                        echo "{bin} $@" >> {path}
                        {script}
                        exit 0
                    fi
                "#,
                path = self.history_file.path().display(),
            });
        }

        let bin_file = self.tmp_dir.child(bin);
        bin_file
            .write_str(&indoc::formatdoc! {
                r#"
                    #! /usr/bin/env bash

                    {scripts}

                    echo "ERROR: Incorrect arguments"
                    echo "ARGS: $@"
                    echo "HISTORY:"
                    cat {path}

                    exit 1
                "#,
                path = self.history_file.path().display(),
            })
            .unwrap();
        let permissions = Permissions::from_mode(0o777);
        std::fs::set_permissions(&bin_file.path(), permissions).unwrap();
    }

    pub fn stub_ok<T>(&mut self, command: T)
    where
        T: AsRef<str>,
    {
        self.stub(command, "")
    }

    pub fn assert_history<I, P>(&self, pred: I)
    where
        I: IntoPathPredicate<P>,
        P: Predicate<Path>,
    {
        self.history_file.assert(pred);
        self.history_file.write_str("").unwrap();
    }
}

fn command_path_str(command: &str) -> String {
    let path = std::process::Command::new("which")
        .arg(command)
        .output()
        .unwrap()
        .stdout;
    let path = String::from_utf8(path).unwrap();
    let path = Path::new(&path).parent().unwrap();

    path.display().to_string()
}
