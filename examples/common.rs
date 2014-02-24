//! Implements some common utilities for the examples

use std::io;
use std::io::BufferedReader;
use lua;

pub fn repl(L: &mut lua::State) {
    let mut stdin = BufferedReader::new(io::stdin());
    let stdout = &mut io::stdout() as &mut io::Writer;
    let stderr = &mut io::stderr() as &mut io::Writer;
    loop {
        L.settop(0); // clear the stack
        let _ = write!(stdout, "> ");
        let _ = stdout.flush();
        let mut line = match stdin.read_line() {
            Ok(line) => line,
            Err(_) => break
        };
        if line.starts_with("=") {
            line = format!("return {}", line.slice_from(1));
        }
        match L.loadbuffer(line, "=stdin") {
            Ok(_) => (),
            Err(err) => { let _ = writeln!(stderr, "{}", err.to_str()); continue; }
        }
        match L.pcall(0, lua::MULTRET, 0) {
            Ok(_) => (),
            Err(_) => {
                match L.tostring(-1) {
                    Some(msg) => { let _ = writeln!(stderr, "{}", msg); }
                    None => { let _ = writeln!(stderr, "(error object is not a string)"); }
                }
            }
        }
        if L.gettop() > 0 {
            L.getglobal("print");
            L.insert(1);
            let nargs = L.gettop()-1;
            match L.pcall(nargs, 0, 0) {
                Ok(_) => (),
                Err(_) => {
                    let _ = writeln!(stderr, "error calling 'print' ({})", L.describe(-1));
                    continue;
                }
            }
        }
    }
}
