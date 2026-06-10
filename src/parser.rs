use std::iter::Peekable;
use std::str::Chars;

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

/// A lexical token produced by the tokenizer, before we know command structure.
#[derive(Debug, PartialEq)]
enum Token {
    Word(String),
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
}

pub fn parse(input: &str) -> Result<ParsedInput, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Ok(ParsedInput::Empty);
    }

    // split the token stream on pipes into per-command groups
    let mut commands: Vec<Command> = Vec::new();
    let mut group: Vec<Token> = Vec::new();
    for token in tokens {
        match token {
            Token::Pipe => commands.push(build_command(std::mem::take(&mut group))?),
            other => group.push(other),
        }
    }
    commands.push(build_command(group)?);

    if commands.len() == 1 {
        Ok(ParsedInput::Single(commands.pop().unwrap()))
    } else {
        Ok(ParsedInput::Pipeline(commands))
    }
}

/// Split the raw input into tokens, honoring single and double quotes so that
/// whitespace and operators (`| < > >>`) inside quotes are treated literally.
/// Variable references (`$VAR`, `${VAR}`) are expanded inline everywhere except
/// inside single quotes.
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = input.chars().peekable();

    // the word currently being assembled
    let mut current = String::new();
    let mut has_word = false; // distinguishes an empty word ("") from no word

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' => flush_word(&mut tokens, &mut current, &mut has_word),
            '|' => {
                flush_word(&mut tokens, &mut current, &mut has_word);
                tokens.push(Token::Pipe);
            }
            '<' => {
                flush_word(&mut tokens, &mut current, &mut has_word);
                tokens.push(Token::RedirectIn);
            }
            '>' => {
                flush_word(&mut tokens, &mut current, &mut has_word);
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::RedirectAppend);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            }
            '$' => {
                has_word = true;
                expand_var(&mut chars, &mut current)?;
            }
            '\'' => {
                has_word = true;
                // single quotes are fully literal: no expansion, no operators
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => current.push(ch),
                        None => return Err("syntax error: unterminated single quote".into()),
                    }
                }
            }
            '"' => {
                has_word = true;
                // double quotes are literal except for variable expansion
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('$') => expand_var(&mut chars, &mut current)?,
                        Some(ch) => current.push(ch),
                        None => return Err("syntax error: unterminated double quote".into()),
                    }
                }
            }
            _ => {
                current.push(c);
                has_word = true;
            }
        }
    }
    flush_word(&mut tokens, &mut current, &mut has_word);

    Ok(tokens)
}

/// Finalize the word being assembled (if any) and push it as a token.
fn flush_word(tokens: &mut Vec<Token>, current: &mut String, has_word: &mut bool) {
    if *has_word {
        tokens.push(Token::Word(std::mem::take(current)));
        *has_word = false;
    }
}

/// Expand a variable reference whose leading `$` has already been consumed,
/// appending its value to `out`. Supports `$NAME` and `${NAME}`. An unset
/// variable, or a lone `$` not followed by a name, is left as literal text.
fn expand_var(chars: &mut Peekable<Chars>, out: &mut String) -> Result<(), String> {
    let braced = chars.peek() == Some(&'{');
    let mut name = String::new();

    if braced {
        chars.next(); // consume '{'
        loop {
            match chars.next() {
                Some('}') => break,
                Some(ch) => name.push(ch),
                None => return Err("syntax error: unterminated '${'".into()),
            }
        }
    } else {
        while let Some(&ch) = chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                name.push(ch);
                chars.next();
            } else {
                break;
            }
        }
    }

    // a lone `$` (or `${}`) with no name: keep it literal
    if name.is_empty() {
        out.push('$');
        if braced {
            out.push_str("{}");
        }
        return Ok(());
    }

    match std::env::var(&name) {
        Ok(value) => out.push_str(&value),
        Err(_) => {
            // unset variable: preserve the original literal text
            out.push('$');
            if braced {
                out.push('{');
                out.push_str(&name);
                out.push('}');
            } else {
                out.push_str(&name);
            }
        }
    }
    Ok(())
}

/// Assemble a single command from its (pipe-free) token group.
fn build_command(tokens: Vec<Token>) -> Result<Command, String> {
    let mut iter = tokens.into_iter();
    let mut program: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    let mut redirect_in: Option<String> = None;
    let mut redirect_out: Option<String> = None;
    let mut redirect_append: Option<String> = None;

    while let Some(token) = iter.next() {
        match token {
            Token::Word(w) => match program {
                None => program = Some(w),
                Some(_) => args.push(w),
            },
            Token::RedirectIn => redirect_in = Some(expect_filename(&mut iter, "<")?),
            Token::RedirectOut => redirect_out = Some(expect_filename(&mut iter, ">")?),
            Token::RedirectAppend => redirect_append = Some(expect_filename(&mut iter, ">>")?),
            Token::Pipe => unreachable!("pipes are split out before build_command"),
        }
    }

    let program = program.ok_or("syntax error: empty command")?;
    Ok(Command {
        program,
        args,
        redirect_in,
        redirect_out,
        redirect_append,
    })
}

/// Consume the next token as a redirection target, or report a syntax error.
fn expect_filename(
    tokens: &mut impl Iterator<Item = Token>,
    operator: &str,
) -> Result<String, String> {
    match tokens.next() {
        Some(Token::Word(filename)) => Ok(filename),
        _ => Err(format!(
            "syntax error: expected filename after '{}'",
            operator
        )),
    }
}
