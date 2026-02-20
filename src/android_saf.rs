// Android MANAGE_EXTERNAL_STORAGE implementation
// Öppnar inställningarna för att be om "Tillåt åtkomst till alla filer"

#[cfg(target_os = "android")]
use ndk_context::android_context;
#[cfg(target_os = "android")]
use jni::objects::{JObject, JValue, JString};
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static VM: OnceLock<jni::JavaVM> = OnceLock::new();

#[cfg(target_os = "android")]
pub fn init_vm(vm: jni::JavaVM) {
    let _ = VM.set(vm);
}

/// Öppna Androids inställningar för MANAGE_EXTERNAL_STORAGE
/// Användaren måste manuellt trycka på "Tillåt åtkomst till alla filer"
#[cfg(target_os = "android")]
pub fn request_all_files_access() -> anyhow::Result<()> {
    let vm = VM.get()
        .ok_or_else(|| anyhow::anyhow!("JNI VM not initialized"))?;

    let mut env = vm.attach_current_thread()
        .map_err(|e| anyhow::anyhow!("Failed to attach thread: {:?}", e))?;

    let ctx = android_context();
    let activity = unsafe { JObject::from_raw(ctx.context() as *mut _) };

    // Skapa Intent för ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION
    let intent_class = env.find_class("android/content/Intent")?;

    // Android 11+ (API 30+): Använd ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION
    let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;
    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)]
    )?;

    // Skapa Uri med "package:" scheme för att identifiera appen
    let uri_class = env.find_class("android/net/Uri")?;
    let package_str = env.new_string("package:com.gnawsoftware.jobseeker")?;
    let uri = env.call_static_method(
        &uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&package_str)]
    )?.l()?;

    // Sätt Uri som data på intentet
    env.call_method(
        &intent,
        "setData",
        "(Landroid/net/Uri;)Landroid/content/Intent;",
        &[JValue::Object(&uri)]
    )?;

    // Starta inställningarna
    env.call_method(
        activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)]
    )?;

    log::info!("SAF: Öppnade inställningar för MANAGE_EXTERNAL_STORAGE");
    Ok(())
}

/// Dummy för non-Android
#[cfg(not(target_os = "android"))]
pub fn request_all_files_access() -> anyhow::Result<()> {
    Err(anyhow::anyhow!("SAF är bara tillgängligt på Android"))
}
