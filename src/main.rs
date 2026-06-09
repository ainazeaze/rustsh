mod builtins;
mod executors;
mod parser;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn main() {
    let mut rl = DefaultEditor::new().expect("failed to init readline");
    let mut history: Vec<String> = Vec::new();

    loop {
        let prompt = format!(
            "{}> ",
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "?".into())
        );

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // add to rustyline history (arrow keys) and our own history vec
                let _ = rl.add_history_entry(&line);
                history.push(line.clone());

                let parsed = parser::parse(&line);

                // check if it's a builtin first
                let is_builtin = match &parsed {
                    parser::ParsedInput::Single(cmd) => builtins::is_builtin(&cmd.program),
                    _ => false,
                };

                let result = if is_builtin {
                    if let parser::ParsedInput::Single(cmd) = parsed {
                        builtins::run(&cmd.program, &cmd.args, &history)
                    } else {
                        Ok(true)
                    }
                } else {
                    executors::execute(parsed)
                };

                match result {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(e) => eprintln!("error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("error: {}", e);
                break;
            }
        }
    }
}
