use crate::parser;
use std::fs::{File, OpenOptions};
use std::process::{Child, Command, Stdio};

pub fn execute(parsed: parser::ParsedInput) -> Result<bool, String> {
    match parsed {
        parser::ParsedInput::Empty => Ok(true),
        parser::ParsedInput::Single(cmd) => run_single(cmd),
        parser::ParsedInput::Pipeline(cmds) => run_pipeline(cmds),
    }
}

fn run_single(cmd: parser::Command) -> Result<bool, String> {
    let mut process = Command::new(&cmd.program);
    process.args(&cmd.args);

    // redirect stdin from a file if < was specified
    if let Some(ref path) = cmd.redirect_in {
        let file = File::open(path).map_err(|e| e.to_string())?;
        process.stdin(Stdio::from(file));
    }

    // redirect stdout to a file if > was specified
    if let Some(ref path) = cmd.redirect_out {
        let file = File::create(path).map_err(|e| e.to_string())?;
        process.stdout(Stdio::from(file));
    }

    // append stdout to a file if >> was specified
    if let Some(ref path) = cmd.redirect_append {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        process.stdout(Stdio::from(file));
    }

    process.status().map_err(|e| e.to_string())?;
    Ok(true)
}

fn run_pipeline(cmds: Vec<parser::Command>) -> Result<bool, String> {
    let mut children: Vec<Child> = Vec::new();
    let mut prev_stdout: Option<Stdio> = None;
    let last = cmds.len() - 1;

    for (i, cmd) in cmds.into_iter().enumerate() {
        let mut process = Command::new(&cmd.program);
        process.args(&cmd.args);

        // connect previous command's stdout to this command's stdin
        if let Some(stdin) = prev_stdout.take() {
            process.stdin(stdin);
        }

        if i == last {
            // last command: inherit stdout (print to terminal), wait for it
            process.status().map_err(|e| e.to_string())?;
        } else {
            // middle command: pipe stdout to next command
            process.stdout(Stdio::piped());
            let mut child = process.spawn().map_err(|e| e.to_string())?;
            // take the pipe out so we can pass it to the next process
            prev_stdout = child.stdout.take().map(Stdio::from);
            children.push(child);
        }
    }

    // wait for all spawned children to finish
    for mut child in children {
        child.wait().map_err(|e| e.to_string())?;
    }

    Ok(true)
}
