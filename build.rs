fn main() {
    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Compile Java classes for Android
    #[cfg(target_os = "android")]
    compile_java_classes();

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

#[cfg(target_os = "android")]
fn compile_java_classes() {
    use std::process::Command;
    use std::path::Path;

    println!("Compiling Java classes for Android...");

    // Find Android SDK
    let android_sdk = std::env::var("ANDROID_SDK_HOME")
        .or_else(|_| std::env::var("ANDROID_HOME"))
        .or_else(|_| std::env::var("ANDROIDSDK_ROOT"))
        .unwrap_or_else(|_| "/opt/android-sdk".to_string());

    let android_jar = Path::new(&android_sdk)
        .join("platforms/android-30/android.jar");

    if !android_jar.exists() {
        eprintln!("Warning: android.jar not found at {:?}", android_jar);
        eprintln!("Java classes will not be compiled");
        return;
    }

    // Create output directory
    let classes_dir = Path::new("android/build/classes");
    std::fs::create_dir_all(classes_dir).unwrap();

    // Compile Java sources
    let java_src = Path::new("android/src/com/gnawsoftware/jobseeker/HttpFetcher.java");
    if java_src.exists() {
        let output = Command::new("javac")
            .args([
                "-d", "android/build/classes",
                "-cp", android_jar.to_str().unwrap(),
                java_src.to_str().unwrap()
            ])
            .output();

        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("Java classes compiled successfully");

                    // Create JAR file for inclusion in APK
                    let jar_output = Command::new("jar")
                        .args([
                            "cf",
                            "android/httpfetcher.jar",
                            "-C", "android/build/classes",
                            "com"
                        ])
                        .output();

                    match jar_output {
                        Ok(jo) if jo.status.success() => {
                            println!("JAR file created: android/httpfetcher.jar");
                            println!("cargo:rustc-env=HTTPFETCHER_JAR=android/httpfetcher.jar");
                        }
                        Ok(jo) => {
                            eprintln!("JAR creation failed: {}", String::from_utf8_lossy(&jo.stderr));
                        }
                        Err(e) => {
                            eprintln!("Failed to create JAR: {}", e);
                        }
                    }
                } else {
                    eprintln!("Java compilation failed: {}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                eprintln!("Failed to run javac: {}", e);
            }
        }
    }
}