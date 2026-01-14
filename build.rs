fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    let slint_path = std::path::Path::new("ui/main.slint");
    if !slint_path.exists() {
        eprintln!("Error: ui/main.slint not found!");
        std::process::exit(1);
    }
    
    println!("Compiling Slint file...");
    match slint_build::compile("ui/main.slint") {
        Ok(_) => println!("Slint compiled successfully"),
        Err(e) => {
            eprintln!("Slint compilation failed: {:?}", e);
            std::process::exit(1);
        }
    }
}