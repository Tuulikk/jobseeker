// Super-Robust JNI HTTP wrapper för Android
// Använder enbart inbyggda systemklasser för maximal kompatibilitet i WayDroid

#[cfg(target_os = "android")]
use jni::objects::{JString, JValueGen};
#[cfg(target_os = "android")]
use log::{info, error};
#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static JAVA_VM: OnceLock<jni::JavaVM> = OnceLock::new();

#[cfg(target_os = "android")]
pub fn set_java_vm(vm: jni::JavaVM) {
    let _ = JAVA_VM.set(vm);
}

// Dummy for compatibility
#[cfg(target_os = "android")]
pub fn set_fetcher_class(_class_ref: jni::objects::GlobalRef) {}

#[cfg(target_os = "android")]
pub fn http_get(url: &str) -> anyhow::Result<String> {
    let vm = JAVA_VM.get().ok_or_else(|| anyhow::anyhow!("JavaVM not initialized"))?;

    // Attach current thread to JVM
    let mut env = vm.attach_current_thread_as_daemon()
        .map_err(|e| anyhow::anyhow!("Failed to attach thread: {:?}", e))?;

    info!("JNI: Fetching {} via HttpURLConnection (Standard API)", url);

    // 1. Create URL object
    let url_class = env.find_class("java/net/URL")
        .map_err(|e| anyhow::anyhow!("Could not find java.net.URL: {:?}", e))?;
    let url_str = env.new_string(url)?;
    let url_obj = env.new_object(&url_class, "(Ljava/lang/String;)V", &[JValueGen::Object(&url_str)])?;

    // 2. Open connection
    let conn_obj = env.call_method(&url_obj, "openConnection", "()Ljava/net/URLConnection;", &[])?.l()?;

    // Set Host header manually since we use raw IP
    let host_key = env.new_string("Host")?;
    let host_val = env.new_string("jobsearch.api.jobtechdev.se")?;
    env.call_method(&conn_obj, "setRequestProperty", "(Ljava/lang/String;Ljava/lang/String;)V", 
        &[JValueGen::Object(&host_key), JValueGen::Object(&host_val)])?;

    // 3. Set timeouts
    env.call_method(&conn_obj, "setConnectTimeout", "(I)V", &[JValueGen::Int(15000)])?;
    env.call_method(&conn_obj, "setReadTimeout", "(I)V", &[JValueGen::Int(30000)])?;

    // 4. Get response code
    let response_code = env.call_method(&conn_obj, "getResponseCode", "()I", &[])?.i()?;
    info!("JNI HTTP: Server returned {}", response_code);

    if response_code != 200 {
        return Err(anyhow::anyhow!("HTTP error {}", response_code));
    }

    // 5. Read response
    let is = env.call_method(&conn_obj, "getInputStream", "()Ljava/io/InputStream;", &[])?.l()?;
    let isr_class = env.find_class("java/io/InputStreamReader")?;
    let isr_obj = env.new_object(&isr_class, "(Ljava/io/InputStream;)V", &[JValueGen::Object(&is)])?;
    
    let br_class = env.find_class("java/io/BufferedReader")?;
    let br_obj = env.new_object(&br_class, "(Ljava/io/Reader;)V", &[JValueGen::Object(&isr_obj)])?;

    let sb_class = env.find_class("java/lang/StringBuilder")?;
    let sb_obj = env.new_object(&sb_class, "()V", &[])?;

    loop {
        let line_obj = env.call_method(&br_obj, "readLine", "()Ljava/lang/String;", &[])?.l()?;
        if line_obj.is_null() {
            break;
        }
        let line = JString::from(line_obj);
        env.call_method(&sb_obj, "append", "(Ljava/lang/String;)Ljava/lang/StringBuilder;", &[JValueGen::Object(&line)])?;
    }

    let result_obj = env.call_method(&sb_obj, "toString", "()Ljava/lang/String;", &[])?.l()?;
    let result_jstr = JString::from(result_obj);
    let result_rust: String = env.get_string(&result_jstr)?.into();

    info!("JNI HTTP: Success, received {} bytes", result_rust.len());
    Ok(result_rust)
}

#[cfg(not(target_os = "android"))]
pub fn http_get(_url: &str) -> anyhow::Result<String> {
    anyhow::bail!("Android JNI HTTP only available on Android");
}
