use amp_core::parquet::{read_address_parquet, read_db_parquet, read_local_parquet, build_local_parquet};
use amp_core::structs::{AdressClean, LocalData, OutputData};
use anyhow::Result;
use anyhow::Context;
use std::fs::File;
use std::fs;
use std::path::PathBuf;
#[cfg(target_os = "android")]
pub fn get_dir() -> anyhow::Result<PathBuf> {
    use jni::JNIEnv;
    use jni::objects::{JObject, JString};
    let (tx, rx) = std::sync::mpsc::channel();
    fn run(env: &mut JNIEnv<'_>, activity: &JObject<'_>) -> anyhow::Result<PathBuf> {
        let files_dir = env
            .call_method(activity, "getFilesDir", "()Ljava/lang/String;", &[])?
            .l()?;
        let files_dir: JString<'_> = env
            .call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?
            .into();
        let files_dir: String = env.get_string(&files_dir)?.into();
        Ok(PathBuf::from(files_dir))
    }
    dioxus::mobile::wry::prelude::dispatch(move |env, activity, _webview| {
        tx.send(run(env, activity)).unwrap()
    });
    rx.recv().unwrap()
}
#[cfg(not(target_os = "android"))]
pub fn get_dir() -> Result<PathBuf> {
    Ok(PathBuf::from("./android/assets/data/db.parquet"))
}
/// Read binary parquet data
pub fn read_db_data() -> Result<Vec<OutputData>> {
    let storage_dir = get_dir()?;
    let file_path = storage_dir.join("./android/assets/data/db.parquet");
    let file =
        File::open(&file_path).map_err(|e| anyhow::anyhow!("Failed to open db.parquet: {}", e))?;
    read_db_parquet(file)
}
/// Read binary parquet data
pub fn read_address_data() -> Result<Vec<AdressClean>> {
    let storage_dir = get_dir()?;
    let file_path = storage_dir.join("./android/assets/data/address.parquet");
    let file = File::open(&file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open address.parquet: {}", e))?;
    read_address_parquet(file)
}
/// Read binary parquet data
pub fn read_local_data() -> Result<Vec<LocalData>> {
    let storage_dir = get_dir()?;
    let file_path = storage_dir.join("./android/assets/data/local.parquet");
    let file = File::open(&file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open local.parquet: {}", e))?;
    read_local_parquet(file)
}
/// Write addresses to persistent storage
pub fn write_local_to_device(_local: &Vec<LocalData>) -> Result<()> {
    #[cfg(target_os = "android")]
    {
        let storage_dir = get_dir()?;
        fs::create_dir_all(&storage_dir)?;
        let file_path = storage_dir.join("local.parquet");
        let serialized = build_local_parquet(_local.clone())?;
        let temp_path = storage_dir.join("local.parquet.tmp");
        fs::write(&temp_path, serialized).context("Failed to write temp file")?;
        fs::rename(&temp_path, &file_path).context("Failed to rename temp file")?;
        Ok(())
    }
    #[cfg(not(target_os = "android"))]
    {
        Ok(())
    }
}
