use clap::CommandFactory;
use clap_complete::{generate, Shell as ClapShell};
use std::io;

use crate::cli::{Cli, Shell};

pub fn execute(shell: &Shell) {
    let mut cmd = Cli::command();
    let clap_shell = match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
    };
    generate(clap_shell, &mut cmd, "clam", &mut io::stdout());
}
