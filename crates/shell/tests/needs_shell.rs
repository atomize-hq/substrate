use proptest::prelude::*;
use substrate_shell::needs_shell;

proptest! {
    #[test]
    fn needs_shell_never_panics(input in "\\PC*") {
        let _ = needs_shell(&input);
    }

    #[test]
    fn simple_commands_dont_need_shell(
        cmd in "[a-zA-Z][a-zA-Z0-9_-]*",
        args in prop::collection::vec("[a-zA-Z0-9_.-]+", 0..5)
    ) {
        let command = if args.is_empty() {
            cmd
        } else {
            format!("{} {}", cmd, args.join(" "))
        };

        prop_assert!(!needs_shell(&command));
    }

    #[test]
    fn pipes_always_need_shell(
        left_cmd in "[a-zA-Z]+",
        right_cmd in "[a-zA-Z]+"
    ) {
        let command = format!("{} | {}", left_cmd, right_cmd);
        prop_assert!(needs_shell(&command));
    }

    #[test]
    fn redirections_need_shell(
        cmd in "[a-zA-Z]+",
        file in "[a-zA-Z0-9._-]+",
        redirect in prop::sample::select(vec![">", ">>", "<", "2>", "&>"])
    ) {
        let command = format!("{} {} {}", cmd, redirect, file);
        prop_assert!(needs_shell(&command));
    }

    #[test]
    fn logical_operators_need_shell(
        left_cmd in "[a-zA-Z]+",
        right_cmd in "[a-zA-Z]+",
        operator in prop::sample::select(vec!["&&", "||", ";"])
    ) {
        let command = format!("{} {} {}", left_cmd, operator, right_cmd);
        prop_assert!(needs_shell(&command));
    }

    #[test]
    fn command_substitution_needs_shell(
        outer_cmd in "[a-zA-Z]+",
        inner_cmd in "[a-zA-Z]+",
        substitution_type in prop::sample::select(vec!["$({})", "`{}`"])
    ) {
        let substitution = substitution_type.replace("{}", &inner_cmd);
        let command = format!("{} {}", outer_cmd, substitution);
        prop_assert!(needs_shell(&command));
    }

    #[test]
    fn background_processes_need_shell(cmd in "[a-zA-Z]+") {
        let command = format!("{} &", cmd);
        prop_assert!(needs_shell(&command));
    }

    #[test]
    fn stderr_redirections_need_shell(
        cmd in "[a-zA-Z]+",
        stderr_redirect in prop::sample::select(vec!["2>&1", "1>&2", "2>"])
    ) {
        let command = format!("{} {}", cmd, stderr_redirect);
        prop_assert!(needs_shell(&command));
    }
}
