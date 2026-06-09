use std::env;

pub fn is_builtin(program: &str) -> bool {
    matches!(program, "cd" | "pwd" | "exit" | "history")
}

pub fn run(program: &str, args: &[String], history: &[String]) -> Result<bool, String> {
    match program {
        "exit" => Ok(false),
        "pwd" => run_pwd(),
        "cd" => run_cd(args),
        "history" => run_history(history),
        _ => Err(format!("unknown builtin: {}", program)),
    }
}

fn run_pwd() -> Result<bool, String> {
    let current_dir = env::current_dir();
    match current_dir {
        Ok(path) => {
            println!("{}", path.display());
            Ok(true)
        }
        Err(_) => Err("could not get current directory".into()),
    }
}

fn run_cd(args: &[String]) -> Result<bool, String> {
    if args.len() == 0 {
        let home_dir = env::var("HOME");
        match home_dir {
            Ok(home) => {
                env::set_current_dir(home).map_err(|e| e.to_string())?;
                Ok(true)
            }
            Err(_) => Err(String::from("Error")),
        }
    } else {
        let dest_dir = &args[0];
        env::set_current_dir(dest_dir).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

fn run_history(history: &[String]) -> Result<bool, String> {
    let history_idx = history.iter().enumerate();
    for (idx, command) in history_idx {
        println!("{} {}", idx, command);
    }
    Ok(true)
}
