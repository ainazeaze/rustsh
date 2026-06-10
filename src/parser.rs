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

pub fn parse(input: &str) -> Result<ParsedInput, String> {
    if input.trim().is_empty() {
        return Ok(ParsedInput::Empty);
    }
    let mut commands: Vec<Command> = input
        .split('|')
        .map(parse_command)
        .collect::<Result<_, _>>()?;
    if commands.len() == 1 {
        Ok(ParsedInput::Single(commands.pop().unwrap()))
    } else {
        Ok(ParsedInput::Pipeline(commands))
    }
}

fn parse_command(segment: &str) -> Result<Command, String> {
    let mut split_peekable = segment.split_whitespace().peekable();
    let mut redirect_in: Option<String> = None;
    let mut redirect_out: Option<String> = None;
    let mut redirect_append: Option<String> = None;

    let program: String = match split_peekable.next() {
        Some(token) => expand_var(token.to_string()),
        None => return Err("syntax error: empty command".into()),
    };

    let mut args: Vec<String> = Vec::new();
    while let Some(token) = split_peekable.next() {
        match token {
            "<" => redirect_in = Some(expect_target(&mut split_peekable, token)?),
            ">" => redirect_out = Some(expect_target(&mut split_peekable, token)?),
            ">>" => redirect_append = Some(expect_target(&mut split_peekable, token)?),
            _ => args.push(expand_var(token.to_string())),
        }
    }

    Ok(Command {
        program,
        args,
        redirect_in,
        redirect_out,
        redirect_append,
    })
}

/// Consume the next token as a redirection target, or report a syntax error.
fn expect_target<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    operator: &str,
) -> Result<String, String> {
    match tokens.next() {
        Some(target) => Ok(target.to_string()),
        None => Err(format!(
            "syntax error: expected filename after '{}'",
            operator
        )),
    }
}

fn expand_var(token: String) -> String {
    if token.starts_with('$') {
        let name = &token.strip_prefix('$').unwrap_or_default(); // strip the '$'
        std::env::var(name).unwrap_or(token.clone()) // look up, fall back to original if not found
    } else {
        token
    }
}
