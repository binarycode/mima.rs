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
    default_flows: HashMap<String, String>,
    history_file: ChildPath,
    tmp_dir: TempDir,
    bash_path: String,
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
        let default_flows = HashMap::new();

        let history_file = tmp_dir.child("history");
        history_file.touch().unwrap();

        let bash_path = which("bash");

        let env = Self {
            bin_path_env,
            config_file,
            flows,
            default_flows,
            history_file,
            tmp_dir,
            bash_path,
        };

        env.create_binary("which", "exit 0");

        env
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
        let script = script.as_ref().to_owned();

        let mut splitter = command.splitn(2, " ");
        let binary = splitter.next().unwrap();
        let arguments = splitter.next().unwrap().to_owned();

        let flows = self.flows.entry(binary.to_owned()).or_default();
        flows.insert(arguments, script);

        self.write_binary_stubs(binary);
    }

    pub fn stub_ok<T>(&mut self, command: T)
    where
        T: AsRef<str>,
    {
        self.stub(command, "")
    }

    pub fn stub_default<T, U>(&mut self, binary: T, script: U)
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let binary = binary.as_ref();
        let script = script.as_ref().to_owned();

        self.default_flows.insert(binary.to_owned(), script);

        self.write_binary_stubs(binary);
    }

    pub fn stub_default_ok<T>(&mut self, binary: T)
    where
        T: AsRef<str>,
    {
        self.stub_default(binary, "")
    }

    pub fn create_binary<T, U>(&self, binary: T, script: U)
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let binary = binary.as_ref();
        let binary = self.tmp_dir.child(binary);
        let script = script.as_ref();
        binary
            .write_str(&indoc::formatdoc! {
                "
                    #!{bash_path}
                    {script}
                ",
                bash_path = self.bash_path
            })
            .unwrap();
        let permissions = Permissions::from_mode(0o777);
        std::fs::set_permissions(&binary.path(), permissions).unwrap();
    }

    fn write_binary_stubs<T>(&self, binary: T)
    where
        T: AsRef<str>,
    {
        let binary = binary.as_ref();

        let history_path = self.history_file.path().display();

        let mut script = "".to_owned();

        if let Some(flows) = self.flows.get(binary) {
            for (arguments, flow_script) in flows {
                script.push_str(&indoc::formatdoc! {r#"
                    if [[ "$@" == "{arguments}" ]]; then
                        echo "{binary} $@" >> {history_path}
                        {flow_script}
                        exit 0
                    fi
                "#});
            }
        }

        if let Some(flow_script) = self.default_flows.get(binary) {
            script.push_str(&indoc::formatdoc! {r#"
                echo "{binary} $@" >> {history_path}
                {flow_script}
            "#});
        } else {
            script.push_str(&indoc::formatdoc! {r#"
                echo "ERROR: Incorrect arguments"
                echo "ARGS: $@"
                echo "HISTORY:"
                cat {history_path}

                exit 1
            "#});
        }

        self.create_binary(binary, script);
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

fn which(command: &str) -> String {
    let path = std::process::Command::new("which")
        .arg(command)
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(path).unwrap()
}

fn command_path_str(command: &str) -> String {
    let path = which(command);
    let path = Path::new(&path).parent().unwrap();

    path.display().to_string()
}
