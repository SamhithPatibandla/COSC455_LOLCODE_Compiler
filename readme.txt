Samhith Patibandla
LOLCODE Markdown Compiler
COSC 455 - Project 1

PROJECT DESCRIPTION
This project implements a LOLCODE Markdown compiler using Rust. The compiler performs lexical analysis, recursive-descent syntax analysis, static-semantic checking, and HTML generation.

TARGET OPERATING SYSTEM
Windows

PROGRAMMING LANGUAGE
Rust

COMPILATION INSTRUCTIONS
1. Open a terminal in the project directory.
2. Ensure Rust and Cargo are installed.
3. Run: cargo build --release
4. The executable will be generated at target\release\lolcompiler.exe.

EXECUTION INSTRUCTIONS
1. Open Command Prompt or PowerShell in the project directory.
2. Run: .\bin\lolcompiler.exe test\Test2.lol
3. The compiler checks the LOLCODE source for lexical, syntax, and semantic errors.
4. If compilation succeeds, an HTML file is generated in the same directory as the input file and opened in the default browser.
5. If compilation fails, an error message is displayed and no new HTML file is generated.

SUPPORTED FEATURES
- Case-insensitive LOLCODE annotations
- Head and title processing
- Comments
- Paragraphs
- Bold text
- Lists and list items
- Newline annotations
- Hyperlinks
- Variable definitions and references
- Static scoping and variable shadowing
- Lexical, syntax, and semantic error detection

TESTING
The compiler was tested against the 10 instructor-provided test cases and includes Rust unit tests.

GITHUB REPOSITORY
[Insert your public GitHub repository URL here]

AI ASSISTANCE
[Add the required 2-3 sentences describing AI assistance and the associated transcript information before submission.]
