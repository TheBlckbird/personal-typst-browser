use std::process::Command;

static FILE: &str = "./o/file.typ";
static OUTPUT: &str = "./out/output.svg";
static ROOT: &str = ".";

fn main() {
    Command::new("typst")
        .arg("compile")
        .arg(FILE)
        .args(["--root", ROOT])
        .arg(OUTPUT)
        .output()
        .unwrap();
}
