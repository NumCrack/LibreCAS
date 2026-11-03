use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    loop {
        write!(stdout, ">>> ")?;
        stdout.flush()?;
        input.clear();

        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        match librecas::execute(&input) {
            Ok(output) => writeln!(stdout, "{output}")?,
            Err(error) => writeln!(stdout, "{error}")?,
        }
    }

    Ok(())
}