// HTTP wrapper för Android som använder Android's inbyggda HttpURLConnection via JNI
// Detta anropar HttpFetcher.java för att utföra nätverksanropet

#[cfg(target_os = "android")]
use jni::objects::{JClass, JString, JValue, GlobalRef};
#[cfg(target_os = "android")]
use log::info;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static JAVA_VM: OnceLock<jni::JavaVM> = OnceLock::new();
#[cfg(target_os = "android")]
static FETCHER_CLASS: OnceLock<GlobalRef> = OnceLock::new();

#[cfg(target_os = "android")]
pub fn set_java_vm(vm: jni::JavaVM) {
    let _ = JAVA_VM.set(vm);
}

#[cfg(target_os = "android")]
pub fn set_fetcher_class(class_ref: GlobalRef) {
    let _ = FETCHER_CLASS.set(class_ref);
}

#[cfg(target_os = "android")]
pub fn http_get(url: &str) -> anyhow::Result<String> {
    let vm = JAVA_VM.get().ok_or_else(|| anyhow::anyhow!("JavaVM not initialized"))?;
    let fetcher_ref = FETCHER_CLASS.get().ok_or_else(|| anyhow::anyhow!("Fetcher class not initialized"))?;

    // Get JNIEnv (attach current thread)
    let mut env = vm.attach_current_thread_as_daemon()
        .map_err(|e| anyhow::anyhow!("Failed to attach thread: {:?}", e))?;

    info!("JNI: Fetching {} via HttpFetcher", url);

    // Use the global reference
    let fetcher_obj = fetcher_ref.as_obj();
    let fetcher_class = unsafe { JClass::from_raw(fetcher_obj.as_raw()) };
    
    let url_jstr = env.new_string(url)
        .map_err(|e| anyhow::anyhow!("Failed to create URL string: {:?}", e))?;
    
    // Call static method HttpFetcher.httpGet(String)
    let result_obj = env.call_static_method(
        &fetcher_class,
        "httpGet",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[JValue::Object(url_jstr.as_ref())]
    ).map_err(|e| anyhow::anyhow!("Failed to call httpGet: {:?}", e))?
    .l()
    .map_err(|_| anyhow::anyhow!("httpGet returned non-object"))?;

    if result_obj.is_null() {
        return Err(anyhow::anyhow!("httpGet returned null (network error)"));
    }

    let result_jstr = JString::from(result_obj);
    let result_rust: String = env.get_string(&result_jstr)
        .map_err(|e| anyhow::anyhow!("Failed to convert to Rust string: {:?}", e))?
        .into();

    Ok(result_rust)
}

#[cfg(not(target_os = "android"))]
pub fn http_get(_url: &str) -> anyhow::Result<String> {
    anyhow::bail!("Android JNI HTTP only available on Android");
}
