use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent");
    
    match slint_build::compile_with_config("ui/main.slint", config) {
        Ok(_) => {
            println!("cargo:warning=Slint compiled successfully");
        }
        Err(e) => {
            eprintln!("Slint compilation error: {:?}", e);
            std::process::exit(1);
        }
    }
}