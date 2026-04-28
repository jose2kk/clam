use clap::CommandFactory;
use clap_complete::{generate, Shell as ClapShell};

use crate::cli::{Cli, Shell};

pub fn execute(shell: &Shell) {
    let mut cmd = Cli::command();
    let clap_shell = match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
    };
    let mut buf = Vec::new();
    generate(clap_shell, &mut cmd, "clam", &mut buf);
    let script = String::from_utf8(buf).expect("clap_complete produced invalid UTF-8");

    let patched = match shell {
        Shell::Bash => patch_bash(&script),
        Shell::Zsh => patch_zsh(&script),
        Shell::Fish => patch_fish(&script),
    };
    print!("{patched}");
}

/// Replace `compgen -f` with dynamic profile list for --profile in run/repair,
/// and add profile completion for positional args in use/remove.
fn patch_bash(script: &str) -> String {
    let profile_compgen =
        r#"COMPREPLY=($(compgen -W "$(clam list --names 2>/dev/null)" -- "${cur}"))"#;

    // Replace --profile file completion with dynamic profile list
    let patched = script.replace(
        r#"--profile)
                    COMPREPLY=($(compgen -f "${cur}"))"#,
        &format!("--profile)\n                    {profile_compgen}"),
    );

    // Add profile completion for `clam use <NAME>` and `clam remove <NAME>`
    let patched = patched.replace(
        r#"clam__use)
            opts="-h --help <NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )"#,
        &format!(
            r#"clam__use)
            opts="-h --help"
            if [[ ${{cur}} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
                return 0
            fi
            {profile_compgen}"#
        ),
    );

    let patched = patched.replace(
        r#"clam__remove)
            opts="-h --force --help <NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )"#,
        &format!(
            r#"clam__remove)
            opts="-h --force --help"
            if [[ ${{cur}} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${{opts}}" -- "${{cur}}") )
                return 0
            fi
            {profile_compgen}"#
        ),
    );

    patched
}

/// Replace `:_default` with `_clam_profiles` helper and inject the function definition.
fn patch_zsh(script: &str) -> String {
    let func_ref = "_clam_profiles";

    // Replace :_default with our helper for --profile flags
    let patched = script.replace("]:PROFILE:_default", &format!("]:PROFILE:{func_ref}"));

    // Replace :_default for positional name args in use/remove (but not add)
    let patched = patched.replace(
        "(use)\n_arguments \"${_arguments_options[@]}\" : \\\n'-h[Print help]' \\\n'--help[Print help]' \\\n':name:_default'",
        &format!("(use)\n_arguments \"${{_arguments_options[@]}}\" : \\\n'-h[Print help]' \\\n'--help[Print help]' \\\n':name:{func_ref}'"),
    );

    let patched = patched.replace(
        "(remove)\n_arguments \"${_arguments_options[@]}\" : \\\n'--force[]' \\\n'-h[Print help]' \\\n'--help[Print help]' \\\n':name:_default'",
        &format!("(remove)\n_arguments \"${{_arguments_options[@]}}\" : \\\n'--force[]' \\\n'-h[Print help]' \\\n'--help[Print help]' \\\n':name:{func_ref}'"),
    );

    // Inject the helper function before the final compdef block
    let helper = r#"(( $+functions[_clam_profiles] )) ||
_clam_profiles() {
    local -a profiles
    profiles=(${(f)"$(clam list --names 2>/dev/null)"})
    _describe -t profiles 'profile' profiles
}

"#;
    let patched = patched.replace(
        "if [ \"$funcstack[1]\" = \"_clam\" ]; then",
        &format!("{helper}if [ \"$funcstack[1]\" = \"_clam\" ]; then"),
    );

    patched
}

/// Add `-f -a "(clam list --names 2>/dev/null)"` for --profile and profile-name arguments.
fn patch_fish(script: &str) -> String {
    let profile_values = r#"-f -a "(clam list --names 2>/dev/null)""#;

    // Replace --profile completions for run and repair with dynamic profile list
    let patched = script.replace(
        r#"complete -c clam -n "__fish_clam_using_subcommand run" -l profile -d 'Use a specific profile (without switching active)' -r"#,
        &format!(r#"complete -c clam -n "__fish_clam_using_subcommand run" -l profile -d 'Use a specific profile (without switching active)' -r {profile_values}"#),
    );

    let patched = patched.replace(
        r#"complete -c clam -n "__fish_clam_using_subcommand repair" -l profile -d 'Repair only a specific profile' -r"#,
        &format!(r#"complete -c clam -n "__fish_clam_using_subcommand repair" -l profile -d 'Repair only a specific profile' -r {profile_values}"#),
    );

    // Add profile completion for `clam use` and `clam remove` positional args
    let patched = patched.replace(
        r#"complete -c clam -n "__fish_clam_using_subcommand use" -s h -l help -d 'Print help'"#,
        &format!("complete -c clam -n \"__fish_clam_using_subcommand use\" {profile_values}\ncomplete -c clam -n \"__fish_clam_using_subcommand use\" -s h -l help -d 'Print help'"),
    );

    let patched = patched.replace(
        r#"complete -c clam -n "__fish_clam_using_subcommand remove" -l force"#,
        &format!("complete -c clam -n \"__fish_clam_using_subcommand remove\" {profile_values}\ncomplete -c clam -n \"__fish_clam_using_subcommand remove\" -l force"),
    );

    patched
}
