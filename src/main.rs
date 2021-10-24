use serde_json::Value;
use std::io::{self, BufRead, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = ColoredWriter::new(StandardStream::stdout(ColorChoice::Always));

    for line in stdin.lock().lines() {
        let line = line?;
        match serde_json::from_str(&line) {
            Ok(json) => {
                write_object(&mut stdout, &json)?;
                stdout.write(TokenKind::None, "\n")?;
            }
            Err(_) => {
                stdout.write_raw(&line)?;
                stdout.write_raw("\n")?;
            }
        }
    }

    Ok(())
}

fn write_value(stdout: &mut ColoredWriter, value: &Value) -> io::Result<()> {
    match value {
        Value::String(string) => stdout.write(TokenKind::String, string),
        Value::Array(array) => {
            stdout.write(TokenKind::None, "[")?;
            for (index, value) in array.iter().enumerate() {
                if index != 0 {
                    stdout.write(TokenKind::None, ", ")?;
                }
                write_value(stdout, value)?;
            }
            stdout.write(TokenKind::None, "]")
        }
        Value::Object(object) => {
            stdout.write(TokenKind::None, "{ ")?;
            write_object(stdout, object)?;
            stdout.write(TokenKind::None, " }")
        }
        _ => stdout.write(TokenKind::Value, &value.to_string()),
    }
}

fn write_object(
    stdout: &mut ColoredWriter,
    object: &serde_json::Map<String, Value>,
) -> io::Result<()> {
    for (index, (key, value)) in object.iter().enumerate() {
        if index != 0 {
            stdout.write_raw(" ")?;
        }
        stdout.write(TokenKind::Key, key)?;
        stdout.write(TokenKind::None, ": ")?;
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
    current: TokenKind,
}

impl ColoredWriter {
    fn new(stream: StandardStream) -> Self {
        ColoredWriter {
            stream,
            current: TokenKind::None,
        }
    }

    fn write(&mut self, kind: TokenKind, string: &str) -> io::Result<()> {
        if self.current != kind {
            self.current = kind;
            let color = match kind {
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
        }
        self.write_raw(string)
    }

    fn write_raw(&mut self, string: &str) -> io::Result<()> {
        self.stream.write_all(string.as_bytes())
    }
}