//! Implements some common utilities for the examples

use std::io;
use lua;

pub fn repl(L: &mut lua::State) {
    let mut stdin = io::buffered::BufferedReader::new(io::stdin());
    let stdout = &mut io::stdout() as &mut io::Writer;
    let stderr = &mut io::stderr() as &mut io::Writer;
    loop {
        L.settop(0); // clear the stack
        write!(stdout, "> ");
        stdout.flush();
        let line = io::io_error::cond.trap(|err| {
            if err.kind == io::EndOfFile {
                write!(stdout, "\n");
                // squelch
            } else {
                fail!(err)
            }
        }).inside(|| stdin.read_line());
        let mut line = if line.is_some() { line.unwrap() } else { break };
        if line.starts_with("=") {
            line = format!("return {}", line.slice_from(1));
        }
        match L.loadbuffer(line, "=stdin") {
            Ok(_) => (),
            Err(err) => { writeln!(stderr, "{}", err.to_str()); continue; }
        }
        match L.pcall(0, lua::MULTRET, 0) {
            Ok(_) => (),
            Err(_) => {
                match L.tostring(-1) {
                    Some(msg) => writeln!(stderr, "{}", msg),
                    None => writeln!(stderr, "(error object is not a string)")
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
                    writeln!(stderr, "error calling 'print' ({})",
                             L.tostring(-1).unwrap_or_default());
                    continue;
                }
            }
        }
    }
}
