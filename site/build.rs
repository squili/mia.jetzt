use sass_rs::OutputStyle;
use std::fs::write;

fn main() {
    println!("cargo:rerun-if-changed=styles.sass");
    println!("cargo:rerun-if-changed=bulma");
    let compiled = sass_rs::compile_file(
        "styles.sass",
        sass_rs::Options {
            output_style: OutputStyle::Compressed,
            ..Default::default()
        },
    )
    .expect("failed compiling sass");
    write("assets/styles.css", compiled).expect("failed writing writing compiled css");
}
