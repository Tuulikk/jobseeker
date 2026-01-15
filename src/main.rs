fn main() {
    #[cfg(not(target_os = "android"))]
    Jobseeker::desktop_main();
}
