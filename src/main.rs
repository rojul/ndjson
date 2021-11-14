use clap::{IntoApp, Parser};
use serde_json::Value;
use std::io::{self, BufRead};
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
        write_line(&mut stdout, &line?)?;
    }

    Ok(())
}

fn write_line<T: WriteColor>(writer: &mut ColoredWriter<T>, line: &str) -> io::Result<()> {
    match serde_json::from_str(line) {
        Ok(Value::Object(object)) if !object.is_empty() => write_object(writer, &object),
        Ok(value) if value.as_array().map_or(false, |array| !array.is_empty()) => {
            write_value(writer, &value)
        }
        _ => writer.write(line),
    }?;
    writer.set_kind(TokenKind::None).write("\n")
}

fn write_value<T: WriteColor>(writer: &mut ColoredWriter<T>, value: &Value) -> io::Result<()> {
    match value {
        Value::String(string) => writer.set_kind(TokenKind::String).write(string),
        Value::Array(array) => {
            writer.set_kind(TokenKind::None).write("[")?;
            for (index, value) in array.iter().enumerate() {
                if index != 0 {
                    writer.set_kind(TokenKind::None).write(", ")?;
                }
                write_value(writer, value)?;
            }
            writer.set_kind(TokenKind::None).write("]")
        }
        Value::Object(object) => {
            if object.is_empty() {
                writer.set_kind(TokenKind::None).write("{}")
            } else {
                writer.set_kind(TokenKind::None).write("{ ")?;
                write_object(writer, object)?;
                writer.set_kind(TokenKind::None).write(" }")
            }
        }
        _ => writer.set_kind(TokenKind::Value).write(&value.to_string()),
    }
}

fn write_object<T: WriteColor>(
    writer: &mut ColoredWriter<T>,
    object: &serde_json::Map<String, Value>,
) -> io::Result<()> {
    for (index, (key, value)) in object.iter().enumerate() {
        if index != 0 {
            writer.write(" ")?;
        }
        writer.set_kind(TokenKind::Key).write(key)?;
        writer.set_kind(TokenKind::None).write(": ")?;
        write_value(writer, value)?;
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

struct ColoredWriter<T: WriteColor> {
    writer: T,
    kind: TokenKind,
    deferred: bool,
}

impl<T: WriteColor> ColoredWriter<T> {
    pub fn new(writer: T) -> Self {
        ColoredWriter {
            writer,
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

    pub fn write(&mut self, string: &str) -> io::Result<()> {
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
                None => self.writer.reset(),
                Some(color) => self
                    .writer
                    .set_color(ColorSpec::new().set_fg(Some(color)).set_intense(true)),
            }?;
            self.deferred = false
        }
        self.writer.write_all(string.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termcolor::Buffer;

    fn format(buffer: Buffer, input: &str) -> String {
        let mut buffer = ColoredWriter::new(buffer);
        write_line(&mut buffer, input).unwrap();
        let mut output = String::from_utf8(buffer.writer.into_inner()).unwrap();
        assert_eq!(output.pop(), Some('\n'));
        output
    }

    #[test]
    fn test_color() {
        assert_eq!(
            format(
                Buffer::ansi(),
                r#"{"null":null,"string":"string","array":[1],"object":{"key":"value"}}"#
            ),
            "[0m[38;5;11mnull[0m: [0m[38;5;10mnull [0m[38;5;11mstring[0m: [0m[38;5;14mstring [0m[38;5;11marray[0m: [[0m[38;5;10m1[0m] [0m[38;5;11mobject[0m: { [0m[38;5;11mkey[0m: [0m[38;5;14mvalue[0m }"
        );
    }

    #[test]
    fn test_unchanged() {
        for s in ["text", "0", "{   }", "[   ]"] {
            assert_eq!(format(Buffer::ansi(), s), s);
        }
    }

    #[test]
    fn test_collections() {
        for (input, output) in [
            (r#"{"key":"value"}"#, "key: value"),
            (r#"["value"]"#, "[value]"),
            (r#"{"array":[],"object":{}}"#, "array: [] object: {}"),
            (r#"[[],{}]"#, "[[], {}]"),
            (
                r#"{"array": ["value"],"object":{"key":"value"}}"#,
                "array: [value] object: { key: value }",
            ),
            (
                r#"[["value"],{"key":"value"}]"#,
                "[[value], { key: value }]",
            ),
        ] {
            assert_eq!(format(Buffer::no_color(), input), output);
        }
    }

    #[test]
    fn test_numbers() {
        for (input, output) in [
            ("0", "0"),
            ("1234567890", "1234567890"),
            ("0.01", "0.01"),
            ("0.00", "0.0"),
            ("1e2", "100.0"),
        ] {
            assert_eq!(
                format(Buffer::no_color(), &format!("[{}]", input)),
                format!("[{}]", output)
            );
        }
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(format(Buffer::no_color(), r#"{"":""}"#), ": ");
        assert_eq!(format(Buffer::no_color(), r#"[""]"#), "[]");
    }
}
