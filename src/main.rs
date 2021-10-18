use serde_json::Value;
use std::io::{self, BufRead, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for line in stdin.lock().lines() {
        let line = line?;
        match serde_json::from_str(&line) {
            Ok(json) => {
                write_object(&mut stdout, &json)?;
                writeln!(&mut stdout)?;
            }
            Err(_) => {
                writeln!(&mut stdout, "{}", line)?;
            }
        }
    }

    Ok(())
}

fn write_value(stdout: &mut StandardStream, value: &Value) -> io::Result<()> {
    match value {
        Value::String(string) => write_colored(stdout, Color::Cyan, string),
        Value::Array(array) => {
            write!(stdout, "[")?;
            for (index, value) in array.iter().enumerate() {
                if index != 0 {
                    write!(stdout, ", ")?;
                }
                write_value(stdout, value)?;
            }
            write!(stdout, "]")
        }
        Value::Object(object) => {
            write!(stdout, "{{ ")?;
            write_object(stdout, object)?;
            write!(stdout, " }}")
        }
        _ => write_colored(stdout, Color::Green, &value.to_string()),
    }
}

fn write_object(
    stdout: &mut StandardStream,
    object: &serde_json::Map<String, Value>,
) -> io::Result<()> {
    for (index, (key, value)) in object.iter().enumerate() {
        if index != 0 {
            write!(stdout, " ")?;
        }
        write_colored(stdout, Color::Yellow, key)?;
        write!(stdout, ": ")?;
        write_value(stdout, value)?;
    }
    Ok(())
}

fn write_colored(stdout: &mut StandardStream, color: Color, string: &str) -> io::Result<()> {
    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_intense(true))?;
    write!(stdout, "{}", string)?;
    stdout.reset()
}
