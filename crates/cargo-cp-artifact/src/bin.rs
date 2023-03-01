mod artifact;
mod cargo;
mod cli;

use cargo::Status;

fn main() {
    // Skip the native binary name (argv[0]).
    if let Status::Failure = cli::run(1) {
        std::process::exit(1);
    }
}
