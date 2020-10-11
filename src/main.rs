mod cargo_config;
mod cli;
mod package_dot_conf;
mod shell_includes;
mod target;
mod whoami;

fn main() {
    cli::Invocation::main()
}
