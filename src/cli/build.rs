use crate::cli::Invocation;
use crate::package_dot_conf::PackageDotConf;
use crate::target::Target;
use clap::Clap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Build an ACAP application
#[derive(Clap)]
pub struct Build {
    /// Which target(s) to build (defaults to all)
    #[clap(short, long, alias = "target")]
    targets: Vec<Target>,

    #[clap(short = 'v', long = "version")]
    show_version: bool,
}

impl Build {
    pub(crate) fn invoke(self, invocation: Invocation) {
        let package_dot_conf = invocation.package_dot_conf();
        let acap_target = invocation.acap_target();
        let version = invocation.package_version();
        let global_options = invocation.global_options();
        let project_source_path = invocation.package_source_path();

        let targets = if self.targets.len() > 0 {
            self.targets.clone()
        } else {
            Target::all().to_vec()
        };

        println!(
            "cargo-acap: building ACAP package `{}` using Docker image {}",
            &package_dot_conf.app_name, &global_options.docker_image
        );

        if self.show_version || global_options.verbose > 0 {
            let mut docker = std::process::Command::new("docker");
            docker.args(&["images", &global_options.docker_image]);
            invocation.run_to_completion(docker);

            let mut docker = invocation.docker_run_command();
            docker.args(&["rustc", "version"]);
            invocation.run_to_completion(docker);
        }

        for target in targets {
            BuildOp {
                invocation: &invocation,
                package_conf: &package_dot_conf,
                cargo_package_name: invocation.cargo_package_name(),
                version: &version,
                project_source_path: &project_source_path,
                acap_target: &acap_target,
                manifest_path: &global_options.manifest_path,
                target,
            }
            .invoke()
        }
    }
}

#[derive(Debug)]
struct BuildOp<'a> {
    invocation: &'a Invocation,
    package_conf: &'a PackageDotConf,
    cargo_package_name: &'a str,
    version: &'a str,
    project_source_path: &'a Path,
    acap_target: &'a Path,
    manifest_path: &'a Path,
    target: Target,
}

impl<'a> BuildOp<'a> {
    pub(crate) fn invoke(&self) {
        eprintln!("cargo-acap: building target {}", self.target.name());
        let built_executable_path = self.cargo_build_in_docker();
        self.copy_executable_with_symbols(&built_executable_path);
        let stripped_executable_path = self.strip_executable(&built_executable_path);
        self.package(&stripped_executable_path)
            .expect("error building package");
    }

    fn cargo_build_in_docker(&self) -> PathBuf {
        let mut docker = self.invocation.docker_run_command();
        docker.args(&[
            "cargo",
            "build",
            "--target",
            self.target.rust_target_triple(),
            "--release",
        ]);

        if self.manifest_path != Path::new("Cargo.toml") {
            docker.arg("--manifest-path");
            docker.arg(&self.manifest_path);
        }

        for _ in 1..self.invocation.global_options().verbose {
            docker.arg("--verbose");
        }

        self.invocation.run_to_completion(docker);

        self.acap_target
            .join(self.target.rust_target_triple())
            .join("release")
            .join(self.cargo_package_name)
    }

    fn artifact_path(&self, suffix: &str) -> PathBuf {
        self.acap_target.join(format!(
            "{}_{}_{}{}",
            &self.package_conf.app_name,
            self.version,
            self.target.name(),
            suffix,
        ))
    }

    fn copy_executable_with_symbols(&self, built_executable_path: &Path) {
        // copy the executable
        let elf_executable_path = self.artifact_path(".elf");
        std::fs::copy(&built_executable_path, &elf_executable_path)
            .expect("error copying built executable");

        if self.invocation.global_options().verbose > 0 {
            let stat = std::fs::metadata(&elf_executable_path).unwrap();
            eprintln!(
                "built executable {} ({} bytes)",
                elf_executable_path.display(),
                stat.len()
            );
        }
    }

    fn strip_executable(&self, built_executable_path: &Path) -> PathBuf {
        let stripped_executable_path = built_executable_path.with_extension("stripped");

        let mut docker = self.invocation.docker_run_command();
        docker.args(&[self.target.docker_objcopy_command(), "--strip-all"]);
        docker.arg(&built_executable_path);
        docker.arg(&stripped_executable_path);

        self.invocation.run_to_completion(docker);

        if self.invocation.global_options().verbose > 1 {
            let stat = std::fs::metadata(&stripped_executable_path).unwrap();
            eprintln!(
                "stripped {} ({} bytes without symbols)",
                stripped_executable_path.display(),
                stat.len()
            );
        }

        stripped_executable_path
    }

    fn package(&self, stripped_executable_path: &Path) -> Result<PathBuf, std::io::Error> {
        let eap = self.artifact_path(".eap");
        let mut file = std::fs::File::create(&eap)?;
        let mut gz = deflate::write::GzEncoder::new(&mut file, deflate::Compression::Default);
        let mut tar = tar::Builder::new(&mut gz);

        let mut package_conf = self.package_conf.clone();

        // write cgi.txt, if any
        {
            let cgi_txt = self.project_source_path.join("cgi.txt");
            match std::fs::File::open(cgi_txt) {
                Ok(mut f) => {
                    tar.append_file("cgi.txt", &mut f)?;
                    package_conf.http_cgi_paths = Some("cgi.txt".into());
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        // ignore
                    } else {
                        return Err(e);
                    }
                }
            }
        };

        // write package.conf
        {
            let package_conf = package_conf.to_string();
            let package_conf_bytes = package_conf.as_bytes();
            tar.append(
                &tar_header(
                    Path::new("package.conf"),
                    package_conf_bytes.len() as _,
                    Some(SystemTime::now()),
                ),
                std::io::Cursor::new(package_conf_bytes),
            )?;
        }

        {
            let mut executable = std::fs::File::open(stripped_executable_path)?;
            tar.append_file(&self.package_conf.app_name, &mut executable)?;
        }

        tar.finish()?;
        drop(tar);
        gz.finish()?;
        file.flush()?;
        drop(file);

        if self.invocation.global_options().verbose > 0 {
            let stat = std::fs::metadata(&eap).unwrap();
            eprintln!("built package {} ({} bytes)", eap.display(), stat.len());
        }

        Ok(eap)
    }
}

fn tar_header(path: &Path, size: u64, mtime: Option<SystemTime>) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(
        mtime
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0),
    );
    header.set_size(size);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    header
}

/*
fn tar_header(path: &Path, size: u64, mtime: Option<SystemTime>) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    let name = b"././@LongLink";
    header.as_gnu_mut().unwrap().name[..name.len()].clone_from_slice(&name[..]);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(
        mtime
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0),
    );
    // + 1 to be compliant with GNU tar
    header.set_size(size + 1);
    header.set_entry_type(tar::EntryType::new(b'L'));
    header.set_cksum();
    header
}
 */
