#[derive(Debug)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub redirect_in: Option<String>,     // < filename
    pub redirect_out: Option<String>,    // > filename
    pub redirect_append: Option<String>, // >> filename
}

#[derive(Debug)]
pub enum ParsedInput {
    Empty,
    Single(Command),
    Pipeline(Vec<Command>), // for cmd1 | cmd2 | cmd3
}

pub fn parse(input: &str) -> ParsedInput {
    if input.trim().is_empty() {
        return ParsedInput::Empty;
    }
    let mut commands: Vec<Command> = input.split("|").map(|s| parse_command(s)).collect();
    if commands.len() == 1 {
        ParsedInput::Single(commands.pop().unwrap())
    } else {
        ParsedInput::Pipeline(commands)
    }
}

fn parse_command(segment: &str) -> Command {
    let mut split_peekable = segment.split_whitespace().peekable();
    let mut redirect_in: Option<String> = None;
    let mut redirect_out: Option<String> = None;
    let mut redirect_append: Option<String> = None;
    let program: String = expand_var(split_peekable.next().unwrap().to_string());

    let mut args: Vec<String> = Vec::new();
    while let Some(token) = split_peekable.next() {
        match token {
            "<" => redirect_in = Some(split_peekable.next().unwrap().to_string()),
            ">" => redirect_out = Some(split_peekable.next().unwrap().to_string()),
            ">>" => redirect_append = Some(split_peekable.next().unwrap().to_string()),
            _ => args.push(expand_var(token.to_string())),
        }
    }

    Command {
        program: program,
        args: args,
        redirect_in: redirect_in,
        redirect_out: redirect_out,
        redirect_append: redirect_append,
    }
}

fn expand_var(token: String) -> String {
    if token.starts_with('$') {
        let name = &token[1..]; // strip the '$'
        std::env::var(name).unwrap_or(token.clone()) // look up, fall back to original if not found
    } else {
        token
    }
}
