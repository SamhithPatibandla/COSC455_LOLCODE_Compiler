mod compiler;
mod lexer;
mod parser;

use compiler::{Compiler, LolCompiler};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// Compile one .lol file into HTML in the same folder.
fn compile_file(filename: &str) -> Result<(), String> {
    let input = Path::new(filename);

    // The assignment requires exactly one .lol file as input.
    if input.extension().and_then(|ext| ext.to_str()) != Some("lol") {
        return Err("Usage error: input must have a .lol extension".to_string());
    }

    // The output basename must match the input basename.
    let output = input.with_extension("html");

    // Remove stale output so an invalid source cannot appear to have compiled.
    // This happens before reading the input, tokenizing, or parsing.
    if output.exists() {
        fs::remove_file(&output).map_err(|e| {
            format!("Cannot remove previous HTML '{}': {}", output.display(), e)
        })?;
    }

    // Read the whole file and compile; no HTML is written unless all checks pass.
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Cannot read '{}': {}", filename, e))?;
    let mut compiler = LolCompiler::new();
    compiler.compile(&source);
    let html = compiler.html().map_err(str::to_string)?;

    fs::write(&output, html)
        .map_err(|e| format!("Cannot write '{}': {}", output.display(), e))?;
    println!("Created {}", output.display());

    // Browser launch is a convenience; the HTML file remains valid if it fails.
    let absolute = output.canonicalize().unwrap_or(output.clone());
    #[cfg(target_os = "windows")]
    let launch = Command::new("cmd")
        .arg("/C").arg("start").arg("").arg(&absolute).spawn();
    #[cfg(target_os = "macos")]
    let launch = Command::new("open").arg(&absolute).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let launch = Command::new("xdg-open").arg(&absolute).spawn();

    if let Err(error) = launch {
        eprintln!("HTML created, but browser could not be opened: {}", error);
    }
    Ok(())
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 2 {
        eprintln!("Usage: lolcompiler <input.lol>");
        std::process::exit(1);
    }
    if let Err(error) = compile_file(&arguments[1]) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}
