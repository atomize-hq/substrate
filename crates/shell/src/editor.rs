use std::borrow::Cow;
use std::fs::File;
use std::path::PathBuf;

use anyhow::{Context, Result};
use reedline::{
    default_emacs_keybindings, ColumnarMenu, Completer, Emacs, ExampleHighlighter, ExternalPrinter,
    FileBackedHistory, MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch,
    PromptHistorySearchStatus, Reedline, ReedlineMenu, Suggestion, ValidationResult, Validator,
};

use crate::ShellConfig;

const HISTORY_CAPACITY: usize = 100_000;
const PRINTER_CAPACITY: usize = 256;

pub(crate) struct EditorSetup {
    pub line_editor: Reedline,
    pub printer: ExternalPrinter<String>,
}

pub(crate) fn build_editor(config: &ShellConfig) -> Result<EditorSetup> {
    let history_path = history_path()?;
    let history = FileBackedHistory::with_file(HISTORY_CAPACITY, history_path.clone())
        .context("error configuring history file")?;

    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        reedline::KeyModifiers::NONE,
        reedline::KeyCode::Tab,
        reedline::ReedlineEvent::Menu("completion_menu".to_string()),
    );
    keybindings.add_binding(
        reedline::KeyModifiers::CONTROL,
        reedline::KeyCode::Char('l'),
        reedline::ReedlineEvent::ClearScreen,
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let completer = Box::new(SubstrateCompleter::new(config));

    let transient_prompt = Box::new(SubstratePrompt::new(config.ci_mode));

    let printer = ExternalPrinter::<String>::new(PRINTER_CAPACITY);
    let printer_handle = printer.clone();

    let line_editor = Reedline::create()
        .with_history(Box::new(history))
        .with_edit_mode(edit_mode)
        .with_completer(completer)
        .with_highlighter(Box::new(ExampleHighlighter::default()))
        .with_validator(Box::new(SubstrateValidator))
        .with_transient_prompt(transient_prompt)
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default().with_name("completion_menu"),
        )))
        .with_external_printer(printer);

    Ok(EditorSetup {
        line_editor,
        printer: printer_handle,
    })
}

pub(crate) fn make_prompt(ci_mode: bool) -> SubstratePrompt {
    SubstratePrompt::new(ci_mode)
}

#[derive(Clone)]
pub(crate) struct SubstratePrompt {
    ci_mode: bool,
}

impl SubstratePrompt {
    pub fn new(ci_mode: bool) -> Self {
        Self { ci_mode }
    }
}

impl Prompt for SubstratePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        if self.ci_mode {
            Cow::Borrowed("> ")
        } else {
            Cow::Borrowed("substrate> ")
        }
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("::: ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        match history_search.status {
            PromptHistorySearchStatus::Passing => Cow::Borrowed("(history search) "),
            PromptHistorySearchStatus::Failing => Cow::Borrowed("(failing search) "),
        }
    }
}

struct SubstrateCompleter {
    commands: Vec<String>,
}

impl SubstrateCompleter {
    fn new(config: &ShellConfig) -> Self {
        let commands = collect_commands_from_path(&config.original_path);
        Self { commands }
    }
}

impl Completer for SubstrateCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let word = extract_word_at_pos(line, pos);

        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(word))
            .take(100)
            .map(|cmd| Suggestion {
                value: cmd.clone(),
                description: None,
                extra: None,
                span: reedline::Span::new(pos - word.len(), pos),
                append_whitespace: true,
                style: None,
            })
            .collect()
    }
}

fn history_path() -> Result<PathBuf> {
    let path = dirs::home_dir()
        .map(|p| p.join(".substrate_history"))
        .unwrap_or_else(|| PathBuf::from(".substrate_history"));

    if !path.exists() {
        File::create(&path).context("failed to create history file")?;
    }

    Ok(path)
}

fn collect_commands_from_path(path: &str) -> Vec<String> {
    let mut commands = Vec::new();
    for dir in path.split(path_separator()) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && is_executable(&metadata) {
                        if let Some(name) = entry.file_name().to_str() {
                            commands.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    commands.sort();
    commands.dedup();
    commands
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    true
}

fn extract_word_at_pos(line: &str, pos: usize) -> &str {
    let start = line[..pos]
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    &line[start..pos]
}

fn path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

#[derive(Default)]
struct SubstrateValidator;

impl Validator for SubstrateValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.trim().is_empty() {
            return ValidationResult::Complete;
        }

        let mut in_single = false;
        let mut in_double = false;
        let mut in_backticks = false;
        let mut escape = false;
        let mut subshell_depth = 0usize;
        let mut brace_stack: Vec<char> = Vec::new();
        let mut pending_operator = false;

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if escape {
                escape = false;
                if pending_operator && !ch.is_whitespace() {
                    pending_operator = false;
                }
                continue;
            }

            if pending_operator && !ch.is_whitespace() {
                pending_operator = false;
            }

            if ch == '\\' && !in_single {
                escape = true;
                continue;
            }

            if ch == '$' && !in_single && !in_backticks {
                if let Some('(') = chars.peek().copied() {
                    chars.next();
                    subshell_depth += 1;
                    continue;
                }
            }

            if in_single {
                if ch == '\'' {
                    in_single = false;
                }
                continue;
            }

            if in_double {
                if ch == '"' {
                    in_double = false;
                }
                continue;
            }

            if in_backticks {
                if ch == '`' {
                    in_backticks = false;
                }
                continue;
            }

            match ch {
                '{' => brace_stack.push('}'),
                '[' => brace_stack.push(']'),
                '(' => brace_stack.push(')'),
                '}' | ']' | ')' => {
                    if brace_stack.last() == Some(&ch) {
                        brace_stack.pop();
                    }
                    if ch == ')' && subshell_depth > 0 {
                        subshell_depth -= 1;
                    }
                }
                '`' => in_backticks = true,
                '\'' => in_single = true,
                '"' => in_double = true,
                '&' => {
                    if let Some(next) = chars.peek().copied() {
                        if next == '&' {
                            chars.next();
                            pending_operator = true;
                            continue;
                        }
                    }
                }
                '|' => {
                    if let Some(next) = chars.peek().copied() {
                        if next == '|' || next == '&' {
                            chars.next();
                            pending_operator = true;
                            continue;
                        }
                    }
                    pending_operator = true;
                    continue;
                }
                _ => {}
            }
        }

        let has_unclosed_quotes = in_single || in_double || in_backticks;
        let has_unmatched_braces = !brace_stack.is_empty() || subshell_depth > 0;
        let has_line_continuation = !in_single && trailing_line_continuation(line);

        if has_unclosed_quotes || has_unmatched_braces || pending_operator || has_line_continuation
        {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

fn trailing_line_continuation(line: &str) -> bool {
    let trimmed = line.trim_end_matches([' ', '\t']);
    if trimmed.is_empty() {
        return false;
    }

    let mut count = 0;
    for ch in trimmed.chars().rev() {
        if ch == '\\' {
            count += 1;
        } else {
            break;
        }
    }
    count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(input: &str) -> ValidationResult {
        SubstrateValidator.validate(input)
    }

    #[test]
    fn detects_unclosed_single_quote() {
        assert!(matches!(validate("echo '"), ValidationResult::Incomplete));
    }

    #[test]
    fn detects_unclosed_double_quote() {
        assert!(matches!(
            validate("echo \"foo"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn detects_trailing_backslash() {
        assert!(matches!(
            validate("echo foo \\"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn detects_pending_pipeline_operator() {
        assert!(matches!(
            validate("echo foo |"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validate("echo foo ||"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validate("echo foo &&"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn completes_valid_command() {
        assert!(matches!(
            validate("echo foo && echo bar"),
            ValidationResult::Complete
        ));
        assert!(matches!(validate("echo '{'"), ValidationResult::Complete));
    }
}
