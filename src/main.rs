use clap::{IntoApp, Parser};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Parser, Debug)]
#[clap(
    version,
    about = "Formats and colorizes newline delimited JSON for better readability.\n\
    The input remains unchanged for non-JSON lines or when stdout isn't a terminal.",
    override_usage = "ndjson < file
    tail -f file | ndjson
    docker logs --tail 100 -f container 2>&1 | ndjson
    kubectl logs --tail 100 -f pod | ndjson"
)]
struct Opt;

fn main() -> io::Result<()> {
    Opt::parse();

    if atty::is(atty::Stream::Stdin) {
        if atty::is(atty::Stream::Stdout) {
            Opt::into_app().print_help()?;
        }
        std::process::exit(1);
    }

    if !atty::is(atty::Stream::Stdout) {
        let mut stdin = io::stdin();
        let mut stdout = io::stdout();
        io::copy(&mut stdin, &mut stdout)?;
        return Ok(());
    }

    let stdin = io::stdin();
    let mut stdout = ColoredWriter::new(StandardStream::stdout(ColorChoice::Always));

    for line in stdin.lock().lines() {
        let line = line?;
        match serde_json::from_str(&line) {
            Ok(Value::Object(object)) if !object.is_empty() => write_object(&mut stdout, &object),
            Ok(value) if value.as_array().map_or(false, |array| !array.is_empty()) => {
                write_value(&mut stdout, &value)
            }
            _ => stdout.write(&line),
        }?;
        stdout.set_kind(TokenKind::None).write("\n")?;
    }

    Ok(())
}

fn write_value(stdout: &mut ColoredWriter, value: &Value) -> io::Result<()> {
    match value {
        Value::String(string) => stdout.set_kind(TokenKind::String).write(string),
        Value::Array(array) => {
            stdout.set_kind(TokenKind::None).write("[")?;
            for (index, value) in array.iter().enumerate() {
                if index != 0 {
                    stdout.set_kind(TokenKind::None).write(", ")?;
                }
                write_value(stdout, value)?;
            }
            stdout.set_kind(TokenKind::None).write("]")
        }
        Value::Object(object) => {
            if object.is_empty() {
                stdout.set_kind(TokenKind::None).write("{}")
            } else {
                stdout.set_kind(TokenKind::None).write("{ ")?;
                write_object(stdout, object)?;
                stdout.set_kind(TokenKind::None).write(" }")
            }
        }
        _ => stdout.set_kind(TokenKind::Value).write(&value.to_string()),
    }
}

fn write_object(
    stdout: &mut ColoredWriter,
    object: &serde_json::Map<String, Value>,
) -> io::Result<()> {
    for (index, (key, value)) in object.iter().enumerate() {
        if index != 0 {
            stdout.write(" ")?;
        }
        stdout.set_kind(TokenKind::Key).write(key)?;
        stdout.set_kind(TokenKind::None).write(": ")?;
        write_value(stdout, value)?;
    }
    Ok(())
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum TokenKind {
    None,
    Key,
    Value,
    String,
}

struct ColoredWriter {
    stream: StandardStream,
    kind: TokenKind,
    deferred: bool,
}

impl ColoredWriter {
    fn new(stream: StandardStream) -> Self {
        ColoredWriter {
            stream,
            kind: TokenKind::None,
            deferred: false,
        }
    }

    pub fn set_kind(&mut self, kind: TokenKind) -> &mut Self {
        if self.kind != kind {
            self.kind = kind;
            self.deferred = true;
        }
        self
    }

    fn write(&mut self, string: &str) -> io::Result<()> {
        if string.is_empty() {
            return Ok(());
        }
        if self.deferred {
            let color = match self.kind {
                TokenKind::None => None,
                TokenKind::Key => Some(Color::Yellow),
                TokenKind::Value => Some(Color::Green),
                TokenKind::String => Some(Color::Cyan),
            };
            match color {
                None => self.stream.reset(),
                Some(color) => self
                    .stream
                    .set_color(ColorSpec::new().set_fg(Some(color)).set_intense(true)),
            }?;
            self.deferred = false
        }
        self.stream.write_all(string.as_bytes())
    }
}
