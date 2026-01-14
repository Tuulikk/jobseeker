fn main() {
    slint_build::print_rustc_link_info();
    slint_build::compile("ui/main.slint").expect("Slint compilation failed");
}