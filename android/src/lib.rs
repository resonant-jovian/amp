use jni::{
    objects::{JClass, JString},
    JNIEnv,
    sys::{jint}
};
use lazy_static::lazy_static;
use std::sync::Mutex;
use amp_core::AppState;

lazy_static! {
    static ref APP_STATE: Mutex<AppState> = Mutex::new(AppState::new());
}

#[unsafe(no_mangle)]
pub extern "C" fn java_com_amp_main_activity_init_app(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    APP_STATE.lock().unwrap().count() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn java_com_amp_main_activity_add_address(
    mut env: JNIEnv,
    _class: JClass,
    address_ptr: JString,
) -> jint {
    match env.get_string(&address_ptr) {
        Ok(address_java) => {
            let address = address_java.to_string_lossy().to_string();
            tracing::info!("Added address: {}", address);
            0
        }
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn java_com_amp_main_activity_get_address_count(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    APP_STATE.lock().unwrap().count() as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn java_com_amp_main_activity_clear_all(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    APP_STATE.lock().unwrap().clear_all();
    0
}
