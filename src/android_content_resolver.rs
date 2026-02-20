// Android ContentResolver wrapper för SAF
// Öppnar filer via ParcelFileDescriptor istället för direkt sökväg

#[cfg(target_os = "android")]
use ndk_context::android_context;
#[cfg(target_os = "android")]
use jni::objects::{JObject, JValue, JString};
#[cfg(target_os = "android")]
use std::sync::OnceLock;
#[cfg(target_os = "android")]
use std::os::unix::io::FromRawFd;

#[cfg(target_os = "android")]
static VM: OnceLock<jni::JavaVM> = OnceLock::new();

#[cfg(target_os = "android")]
pub fn init_vm(vm: jni::JavaVM) {
    let _ = VM.set(vm);
}

/// Öppna en fil via ContentResolver och returnera file descriptor
/// Används för att kringgå MediaProvider-blockering på Android 10+
#[cfg(target_os = "android")]
pub fn open_file_via_content_resolver(file_path: &str) -> anyhow::Result<std::fs::File> {
    let vm = VM.get()
        .ok_or_else(|| anyhow::anyhow!("JNI VM not initialized"))?;

    let mut env = vm.attach_current_thread()
        .map_err(|e| anyhow::anyhow!("Failed to attach thread: {:?}", e))?;

    let ctx = android_context();
    let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };

    // Hämta ContentResolver
    let resolver = env.call_method(
        activity,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[]
    )?.l()?;

    // Konvertera filsökväg till URI (file:// scheme)
    let uri_string = format!("file://{}", file_path);
    let uri_jstring = env.new_string(&uri_string)?;

    let uri_class = env.find_class("android/net/Uri")?;
    let uri = env.call_static_method(
        &uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&uri_jstring)]
    )?.l()?;

    // Öppna filen med ParcelFileDescriptor
    let mode = env.new_string("rw")?;
    let pfd = env.call_method(
        resolver,
        "openFileDescriptor",
        "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
        &[JValue::Object(&uri), JValue::Object(&mode)]
    )?.l()?;

    // Hämta file descriptor
    let fd = env.call_method(pfd, "getFd", "()I", &[])?.i()?;

    // Konvertera till Rust File
    let file = unsafe { std::fs::File::from_raw_fd(fd) };

    log::info!("ContentResolver: Öppnade {} via FD {}", file_path, fd);

    Ok(file)
}

/// Dummy för non-Android
#[cfg(not(target_os = "android"))]
pub fn open_file_via_content_resolver(file_path: &str) -> anyhow::Result<std::fs::File> {
    Err(anyhow::anyhow!("ContentResolver är bara tillgängligt på Android"))
}
