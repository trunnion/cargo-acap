use crate::package_dot_conf::PackageDotConf;
use crate::whoami::whoami;
use clap::Clap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

mod build;
mod targets;

#[derive(Clap)]
#[clap(author, about)]
struct Args {
    #[clap(flatten)]
    global_options: GlobalOptions,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clap)]
pub struct GlobalOptions {
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Path to the application project's Cargo.toml
    #[clap(long, default_value = "Cargo.toml")]
    manifest_path: PathBuf,

    /// `docker` image to use for cross-compiling
    #[clap(long, default_value = "trunnion/cargo-acap")]
    docker_image: String,
}

#[derive(Clap)]
enum Subcommand {
    Build(build::Build),
    Targets(targets::Targets),
}

#[derive(Debug)]
pub struct Invocation {
    global_options: GlobalOptions,
    rustc: cargo::util::Rustc,
    cargo_home: PathBuf,
    workspace_root: PathBuf,
    workspace_target: PathBuf,
    cargo_package: cargo::core::Package,
    acap_target: Mutex<Option<PathBuf>>,
}

/// Process arguments, where `cargo acap …` is treated as `cargo-acap …`
fn cargo_acap_args() -> impl Iterator<Item = OsString> {
    let mut args: Vec<OsString> = std::env::args_os().collect();

    match (args.get(0), args.get(1)) {
        (Some(cargo), Some(acap))
            if cargo.to_string_lossy().contains("cargo") && acap == "acap" =>
        {
            // We were invoked as `cargo acap` or similar
            // Pretend we were invoked as `cargo-acap` -- dropping `acap` -- so that clap matching
            // works as expected
            args.remove(0);
            args[0] = OsString::from(String::from("cargo-acap"));
        }
        _ => {}
    }

    args.into_iter()
}

impl Invocation {
    pub fn main() -> ! {
        let Args {
            mut global_options,
            subcommand,
            ..
        } = Args::parse_from(cargo_acap_args());

        let cargo_config = cargo::Config::default().expect("error constructing `cargo` config");
        let cargo_home = cargo_config.home().as_path_unlocked().to_owned();
        let manifest_path = global_options
            .manifest_path
            .canonicalize()
            .expect("error canonicalizing the manifest path");

        let cargo_workspace = cargo::core::Workspace::new(&manifest_path, &cargo_config)
            .expect("error loading `cargo` workspace");
        let workspace_root = cargo_workspace
            .root()
            .to_owned()
            .canonicalize()
            .expect("error canonicalizing workspace root");
        let workspace_target = {
            let fs = cargo_workspace.target_dir();
            let path = fs.as_path_unlocked();
            std::fs::create_dir_all(path).expect("error creating target/");
            path.canonicalize().expect("error canonicalizing target/")
        };

        let cargo_package = cargo_workspace
            .current()
            .expect("error getting current `cargo` package")
            .clone();

        let rustc = cargo_config
            .load_global_rustc(Some(&cargo_workspace))
            .expect("error loading rustc");

        if global_options.docker_image.contains(':') {
            // use the provided tag
        } else {
            // use rustc's version as the tag
            let image_with_tag = format!("{}:{}", &global_options.docker_image, &rustc.version);
            global_options.docker_image = image_with_tag;
        };

        let invocation = Invocation {
            global_options,
            rustc,
            cargo_home,
            workspace_root,
            workspace_target,
            cargo_package,
            acap_target: Mutex::new(None),
        };

        match subcommand {
            Subcommand::Build(sub) => sub.invoke(invocation),
            Subcommand::Targets(sub) => sub.invoke(invocation),
        };

        std::process::exit(0);
    }

    pub fn global_options(&self) -> &GlobalOptions {
        &self.global_options
    }

    pub fn cargo_package_name(&self) -> &str {
        self.cargo_package.name().as_str()
    }

    pub fn acap_target(&self) -> PathBuf {
        let mut lock = self.acap_target.lock().unwrap();

        if let Some(dir) = lock.as_ref() {
            return dir.clone();
        }

        let acap_target = self.workspace_target.join("acap");
        std::fs::create_dir(&acap_target)
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .expect("error creating target/acap/");

        lock.replace(acap_target.clone());
        acap_target
    }

    pub fn docker_run_command(&self) -> std::process::Command {
        // Start constructing the command
        let mut docker = std::process::Command::new("docker");
        docker.args(&["run", "--rm"]);
        if std::io::stdin().is_tty() {
            docker.arg("--interactive");
            if std::io::stdout().is_tty() {
                docker.arg("--tty");
            }
        }

        // Run with the right uid, gid, and USER env var
        let whoami = whoami();
        docker.args(&["--user", &format!("{}:{}", whoami.uid, whoami.gid)]);
        if let Some(username) = whoami.username.as_ref() {
            docker.args(&["--env", &format!("USER={}", username)]);
        }

        // Mount the root_path at root_path path and use it as the current directory
        docker.args(&[
            "--volume",
            &format!(
                "{}:/{}:Z",
                self.workspace_root.display(),
                self.workspace_root.display()
            ),
            "--workdir",
            &self.cargo_package.root().display().to_string(),
        ]);

        // Mount target_path at /target and tell `cargo` to use it
        docker.args(&[
            "--volume",
            &format!("{}:/target:Z", self.acap_target().display().to_string()),
        ]);
        docker.args(&["--env", "CARGO_TARGET_DIR=/target"]);

        // Mount the cargo home at /.cargo
        docker.args(&[
            "--volume",
            &format!("{}:/.cargo:Z", self.cargo_home.display().to_string()),
        ]);

        if let Ok(value) = std::env::var("DOCKER_OPTS") {
            let opts: Vec<&str> = value.split(' ').collect();
            docker.args(&opts);
        }

        docker.arg(&self.global_options.docker_image);
        docker
    }

    pub(crate) fn package_dot_conf(&self) -> PackageDotConf {
        self.cargo_package.clone().into()
    }

    pub fn run_to_completion(&self, mut command: std::process::Command) {
        if self.global_options.verbose > 1 {
            println!("+ {:?}", &command);
        }

        let exit_status = command
            .spawn()
            .expect("error running command")
            .wait()
            .expect("command failed");
        if !exit_status.success() {
            let code = exit_status.code().expect("code() for failed exit status");
            eprintln!(
                "`cargo acap` failed: `{:?}` returned exit code {}",
                &command, code
            );
            std::process::exit(code);
        }
    }

    pub fn package_source_path(&self) -> PathBuf {
        self.cargo_package.root().join("src")
    }

    pub fn package_version(&self) -> String {
        self.cargo_package.version().to_string()
    }
}

trait StdioExt {
    fn is_tty(&self) -> bool;
}

#[cfg(unix)]
impl StdioExt for std::io::Stdin {
    fn is_tty(&self) -> bool {
        use std::os::unix::io::AsRawFd;
        // Safety: at worst isatty() just sets errno
        unsafe { libc::isatty(self.as_raw_fd()) == 1 }
    }
}

#[cfg(unix)]
impl StdioExt for std::io::Stdout {
    fn is_tty(&self) -> bool {
        use std::os::unix::io::AsRawFd;
        // Safety: at worst isatty() just sets errno
        unsafe { libc::isatty(self.as_raw_fd()) == 1 }
    }
}

#[cfg(windows)]
impl StdioExt for std::io::Stdin {
    fn is_tty(&self) -> bool {
        // FIXME
        false
    }
}

#[cfg(windows)]
impl StdioExt for std::io::Stdout {
    fn is_tty(&self) -> bool {
        // FIXME
        false
    }
}

#[cfg(all(not(unix), not(windows)))]
impl StdioExt for std::io::Stdin {
    fn is_tty(&self) -> bool {
        unimplemented!()
    }
}
#[cfg(all(not(unix), not(windows)))]
impl StdioExt for std::io::Stdout {
    fn is_tty(&self) -> bool {
        unimplemented!()
    }
}
