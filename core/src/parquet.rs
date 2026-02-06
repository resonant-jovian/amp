//! Apache Parquet I/O for parking data persistence.
//!
//! This module handles serialization and deserialization of all data structures
//! to/from Apache Parquet format using Apache Arrow. Parquet provides efficient
//! columnar storage with good compression, ideal for mobile apps with limited storage.
//!
//! # Supported Data Types
//!
//! - [`OutputData`]: Correlated address and parking information
//! - [`LocalData`]: User's saved addresses with active status
//! - [`AdressClean`]: Address data with coordinates
//! - [`SettingsData`]: User preferences for notifications and UI
//! - [`DebugAddress`]: Minimal address entries for testing
//!
//! # File vs. Memory Operations
//!
//! The module provides two patterns:
//! - **File-based**: `read_*_parquet(file)` and `write_*_parquet(data, path)` for desktop/server
//! - **Memory-based**: `read_*_from_bytes(bytes)` and `build_*_parquet(data)` for Android/embedded
//!
//! Memory-based operations are used in Android where Parquet files are bundled as
//! assets or stored in app-private directories.
//!
//! # Schema Definitions
//!
//! Each data type has a corresponding schema function:
//! - [`output_data_schema`]: 10 columns with mixed nullable/non-nullable fields
//! - [`local_data_schema`]: 12 columns including `valid` and `active` flags
//! - [`adress_clean_schema`]: 6 columns with Float64 coordinates
//! - [`settings_data_schema`]: 5 columns for app preferences
//!
//! # Examples
//!
//! ## Writing Data to File
//!
//! ```no_run
//! use amp_core::parquet::write_output_parquet;
//! use amp_core::structs::OutputData;
//!
//! let data = vec![/* OutputData entries */];
//! # let data: Vec<OutputData> = vec![];
//! write_output_parquet(data, "output.parquet")?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Building In-Memory Buffer (Android)
//!
//! ```no_run
//! use amp_core::parquet::build_local_parquet;
//! use amp_core::structs::LocalData;
//!
//! let data = vec![/* LocalData entries */];
//! # let data: Vec<LocalData> = vec![];
//! let parquet_bytes = build_local_parquet(data)?;
//! // Write to Android internal storage or send via JNI
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Reading from Embedded Bytes
//!
//! ```no_run
//! use amp_core::parquet::read_local_parquet_from_bytes;
//!
//! const EMBEDDED_DATA: &[u8] = include_bytes!("data.parquet");
//! let data = read_local_parquet_from_bytes(EMBEDDED_DATA)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! [`OutputData`]: crate::structs::OutputData
//! [`LocalData`]: crate::structs::LocalData
//! [`AdressClean`]: crate::structs::AdressClean
//! [`SettingsData`]: crate::structs::SettingsData
use crate::structs::*;
use anyhow;
use arrow::array::{
    Array, BooleanArray, BooleanBuilder, Float64Builder, UInt8Array, UInt8Builder, UInt64Array,
    UInt64Builder,
};
use arrow::{
    array::{StringArray, StringBuilder},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use bytes::Bytes;
use parquet::{
    arrow::ArrowWriter,
    arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use rust_decimal::prelude::FromPrimitive;
use std::{fs::File, sync::Arc};
/// Schema for [`OutputData`] parquet format.
///
/// Defines 10 columns with mixed nullability:
/// - Non-nullable: `adress`, `gata`, `gatunummer`
/// - Nullable: `postnummer`, `info`, `tid`, `dag`, `taxa`, `antal_platser`, `typ_av_parkering`
///
/// # Column Types
///
/// - String columns: `postnummer`, `adress`, `gata`, `gatunummer`, `info`, `tid`, `taxa`, `typ_av_parkering`
/// - Integer columns: `dag` (UInt8), `antal_platser` (UInt64)
///
/// [`OutputData`]: crate::structs::OutputData
pub fn output_data_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::Utf8, true),
        Field::new("adress", DataType::Utf8, false),
        Field::new("gata", DataType::Utf8, false),
        Field::new("gatunummer", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, true),
        Field::new("tid", DataType::Utf8, true),
        Field::new("dag", DataType::UInt8, true),
        Field::new("taxa", DataType::Utf8, true),
        Field::new("antal_platser", DataType::UInt64, true),
        Field::new("typ_av_parkering", DataType::Utf8, true),
    ]))
}
/// Schema for [`AdressClean`] parquet format.
///
/// Defines 6 columns with coordinate data:
/// - `longitude`, `latitude`: Float64 (non-nullable)
/// - `postnummer`: Utf8 (nullable)
/// - `adress`, `gata`, `gatunummer`: Utf8 (non-nullable)
///
/// Coordinates are stored as Float64 for compatibility with GIS tools,
/// converted from [`Decimal`] during serialization.
///
/// [`AdressClean`]: crate::structs::AdressClean
/// [`Decimal`]: rust_decimal::Decimal
pub fn adress_clean_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("longitude", DataType::Float64, false),
        Field::new("latitude", DataType::Float64, false),
        Field::new("postnummer", DataType::Utf8, true),
        Field::new("adress", DataType::Utf8, false),
        Field::new("gata", DataType::Utf8, false),
        Field::new("gatunummer", DataType::Utf8, false),
    ]))
}
/// Schema for [`LocalData`] parquet format.
///
/// Defines 12 columns including validation and active status:
/// - Non-nullable: `valid`, `active` (Boolean), `adress` (Utf8)
/// - Nullable: All parking-related fields (postnummer, gata, info, etc.)
///
/// This schema extends [`output_data_schema`] with:
/// - `valid`: Whether address was matched in database
/// - `active`: Whether notifications are enabled
///
/// [`LocalData`]: crate::structs::LocalData
pub fn local_data_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("valid", DataType::Boolean, false),
        Field::new("active", DataType::Boolean, false),
        Field::new("postnummer", DataType::Utf8, true),
        Field::new("adress", DataType::Utf8, false),
        Field::new("gata", DataType::Utf8, true),
        Field::new("gatunummer", DataType::Utf8, true),
        Field::new("info", DataType::Utf8, true),
        Field::new("tid", DataType::Utf8, true),
        Field::new("dag", DataType::UInt8, true),
        Field::new("taxa", DataType::Utf8, true),
        Field::new("antal_platser", DataType::UInt64, true),
        Field::new("typ_av_parkering", DataType::Utf8, true),
    ]))
}
/// Extract a StringArray column from a RecordBatch.
///
/// # Errors
///
/// Returns error if:
/// - Column name doesn't exist in schema
/// - Column is not of type StringArray
fn get_string_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
) -> anyhow::Result<&'a StringArray> {
    batch
        .column(batch.schema().index_of(column_name)?)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow::anyhow!("{} column missing or wrong type", column_name))
}
/// Extract a BooleanArray column from a RecordBatch.
///
/// # Errors
///
/// Returns error if column doesn't exist or is not Boolean type.
fn get_boolean_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
) -> anyhow::Result<&'a BooleanArray> {
    batch
        .column(batch.schema().index_of(column_name)?)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| anyhow::anyhow!("{} column missing or wrong type", column_name))
}
/// Extract a UInt8Array column from a RecordBatch.
///
/// Used for reading `dag` (day of month) fields.
///
/// # Errors
///
/// Returns error if column doesn't exist or is not UInt8 type.
fn get_u8_column<'a>(batch: &'a RecordBatch, column_name: &str) -> anyhow::Result<&'a UInt8Array> {
    batch
        .column(batch.schema().index_of(column_name)?)
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| anyhow::anyhow!("{} column missing or wrong type", column_name))
}
/// Extract a UInt64Array column from a RecordBatch.
///
/// Used for reading `antal_platser` (number of parking spots) fields.
///
/// # Errors
///
/// Returns error if column doesn't exist or is not UInt64 type.
fn get_u64_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
) -> anyhow::Result<&'a UInt64Array> {
    batch
        .column(batch.schema().index_of(column_name)?)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| anyhow::anyhow!("{} column missing or wrong type", column_name))
}
/// Get optional string value from StringArray at index.
///
/// Returns `None` if the value is null, `Some(String)` otherwise.
fn get_optional_string(array: &StringArray, index: usize) -> Option<String> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index).to_string())
    }
}
/// Get required string value from StringArray at index.
///
/// Returns empty string if value is null (used for non-nullable schema fields).
fn get_required_string(array: &StringArray, index: usize) -> String {
    if array.is_null(index) {
        String::new()
    } else {
        array.value(index).to_string()
    }
}
/// Get optional u8 value from UInt8Array at index.
///
/// Returns `None` if the value is null.
fn get_optional_u8(array: &UInt8Array, index: usize) -> Option<u8> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index))
    }
}
/// Get optional u64 value from UInt64Array at index.
///
/// Returns `None` if the value is null.
fn get_optional_u64(array: &UInt64Array, index: usize) -> Option<u64> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index))
    }
}
/// Get boolean value from BooleanArray at index with default fallback.
///
/// Returns the default value if the cell is null.
fn get_boolean_with_default(array: &BooleanArray, index: usize, default: bool) -> bool {
    if array.is_null(index) {
        default
    } else {
        array.value(index)
    }
}
/// Create a Parquet reader from a file handle.
///
/// Sets up the Arrow reader for batch-wise reading of Parquet data.
///
/// # Errors
///
/// Returns error if file is not valid Parquet format.
fn create_parquet_reader(
    file: File,
) -> anyhow::Result<impl Iterator<Item = Result<RecordBatch, arrow::error::ArrowError>>> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    Ok(reader)
}
/// Append optional string to StringBuilder.
///
/// Appends null if `value` is `None`, otherwise appends the string value.
fn append_optional_string(builder: &mut StringBuilder, value: &Option<String>) {
    match value {
        Some(v) => builder.append_value(v.clone()),
        None => builder.append_null(),
    }
}
/// Append optional u8 to UInt8Builder.
///
/// Appends null if `value` is `None`.
fn append_optional_u8(builder: &mut UInt8Builder, value: &Option<u8>) {
    match value {
        Some(v) => builder.append_value(*v),
        None => builder.append_null(),
    }
}
/// Append optional u64 to UInt64Builder.
///
/// Appends null if `value` is `None`.
fn append_optional_u64(builder: &mut UInt64Builder, value: &Option<u64>) {
    match value {
        Some(v) => builder.append_value(*v),
        None => builder.append_null(),
    }
}
/// Create ArrowWriter with standard properties.
///
/// Creates a Parquet writer with:
/// - Statistics disabled (for faster writes on mobile)
/// - Default compression (Snappy)
///
/// # Errors
///
/// Returns error if file cannot be created.
fn create_arrow_writer(path: &str, schema: Arc<Schema>) -> anyhow::Result<ArrowWriter<File>> {
    let file = File::create(path).map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))
}
/// Write a single batch and close the writer.
///
/// Helper function to write one RecordBatch and properly close the writer.
///
/// # Errors
///
/// Returns error if write or close operation fails.
fn write_batch_and_close(mut writer: ArrowWriter<File>, batch: RecordBatch) -> anyhow::Result<()> {
    writer
        .write(&batch)
        .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}
/// Minimal address entry for testing and debugging.
///
/// Contains only address string and postal code, used for:
/// - Loading test addresses from debug.parquet
/// - Simulating user address input in tests
/// - Verifying address matching logic
#[derive(Debug, Clone)]
pub struct DebugAddress {
    pub adress: String,
    pub postnummer: String,
}
/// Load debug addresses from embedded bytes.
///
/// Reads a minimal parquet file with `adress` and `postnummer` columns.
/// This simulates a user entering addresses via "Add Address" button.
///
/// The returned entries can then be fuzzy-matched against the static
/// parking database using [`StoredAddress::to_local_data`].
///
/// # Arguments
///
/// * `bytes` - Byte slice containing the debug.parquet file data
///
/// # Returns
///
/// Vector of [`DebugAddress`] entries with address and postal code.
///
/// # Errors
///
/// Returns error if:
/// - Data is not valid Parquet format
/// - Required columns (`adress`, `postnummer`) are missing
///
/// # Examples
///
/// ```no_run
/// use amp_core::parquet::load_debug_addresses;
///
/// const DEBUG_DATA: &[u8] = include_bytes!("../../../android/app/src/main/assets/debug.parquet");
/// let addresses = load_debug_addresses(DEBUG_DATA)?;
/// println!("Loaded {} debug addresses", addresses.len());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// [`StoredAddress::to_local_data`]: crate::structs::StoredAddress::to_local_data
pub fn load_debug_addresses(bytes: &[u8]) -> anyhow::Result<Vec<DebugAddress>> {
    println!(
        "[load_debug_addresses] Loading debug addresses from {} bytes",
        bytes.len(),
    );
    let bytes_obj = Bytes::copy_from_slice(bytes);
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes_obj)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
        let address_array = get_string_column(&batch, "adress")?;
        let postnummer_array = get_string_column(&batch, "postnummer")?;
        for i in 0..batch.num_rows() {
            let address_str = get_required_string(address_array, i);
            let postnummer_str = get_required_string(postnummer_array, i);
            if !address_str.is_empty() {
                println!(
                    "[load_debug_addresses] Loaded: '{}' ({})",
                    address_str, postnummer_str,
                );
                result.push(DebugAddress {
                    adress: address_str,
                    postnummer: postnummer_str,
                });
            }
        }
    }
    println!(
        "[load_debug_addresses] Successfully loaded {} debug addresses",
        result.len(),
    );
    Ok(result)
}
/// Load debug addresses from a file path.
///
/// File-based version of [`load_debug_addresses`] for desktop/server testing.
///
/// # Arguments
///
/// * `path` - Path to the debug.parquet file
///
/// # Returns
///
/// Vector of [`DebugAddress`] entries.
///
/// # Errors
///
/// Returns error if file cannot be opened or is not valid Parquet format.
pub fn load_debug_addresses_from_file(path: &str) -> anyhow::Result<Vec<DebugAddress>> {
    let file = File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open debug parquet file: {}", e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
        let address_array = get_string_column(&batch, "adress")?;
        let postnummer_array = get_string_column(&batch, "postnummer")?;
        for i in 0..batch.num_rows() {
            let address_str = get_required_string(address_array, i);
            let postnummer_str = get_required_string(postnummer_array, i);
            if !address_str.is_empty() {
                result.push(DebugAddress {
                    adress: address_str,
                    postnummer: postnummer_str,
                });
            }
        }
    }
    Ok(result)
}
/// Read [`OutputData`] from a parquet file.
///
/// Loads correlated address and parking information from persistent storage.
/// Typically used for loading the static parking database on app startup.
///
/// # Arguments
///
/// * `file` - Open file handle to parquet file
///
/// # Returns
///
/// Vector of [`OutputData`] entries with all parking information.
///
/// # Errors
///
/// Returns error if file is not valid Parquet or schema doesn't match.
///
/// [`OutputData`]: crate::structs::OutputData
pub fn read_db_parquet(file: File) -> anyhow::Result<Vec<OutputData>> {
    let mut reader = create_parquet_reader(file)?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let postnummer = get_string_column(&batch, "postnummer")?;
        let address = get_string_column(&batch, "adress")?;
        let gata = get_string_column(&batch, "gata")?;
        let gatunummer = get_string_column(&batch, "gatunummer")?;
        let info = get_string_column(&batch, "info")?;
        let dag = get_u8_column(&batch, "dag")?;
        let tid = get_string_column(&batch, "tid")?;
        let taxa = get_string_column(&batch, "taxa")?;
        let antal_platser = get_u64_column(&batch, "antal_platser")?;
        let typ_av_parkering = get_string_column(&batch, "typ_av_parkering")?;
        for i in 0..batch.num_rows() {
            let entry = OutputData {
                postnummer: get_optional_string(postnummer, i),
                adress: get_required_string(address, i),
                gata: get_required_string(gata, i),
                gatunummer: get_required_string(gatunummer, i),
                info: get_optional_string(info, i),
                tid: get_optional_string(tid, i),
                dag: get_optional_u8(dag, i),
                taxa: get_optional_string(taxa, i),
                antal_platser: get_optional_u64(antal_platser, i),
                typ_av_parkering: get_optional_string(typ_av_parkering, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Read [`LocalData`] from a parquet file.
///
/// Loads user's saved addresses with matched parking information.
/// Used for loading saved state when app starts.
///
/// # Arguments
///
/// * `file` - Open file handle to parquet file
///
/// # Returns
///
/// Vector of [`LocalData`] entries with validation and active status.
///
/// # Errors
///
/// Returns error if file is not valid Parquet or schema doesn't match.
///
/// [`LocalData`]: crate::structs::LocalData
pub fn read_local_parquet(file: File) -> anyhow::Result<Vec<LocalData>> {
    let mut reader = create_parquet_reader(file)?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let valid = get_boolean_column(&batch, "valid")?;
        let active = get_boolean_column(&batch, "active")?;
        let postnummer = get_string_column(&batch, "postnummer")?;
        let address = get_string_column(&batch, "adress")?;
        let gata = get_string_column(&batch, "gata")?;
        let gatunummer = get_string_column(&batch, "gatunummer")?;
        let info = get_string_column(&batch, "info")?;
        let dag = get_u8_column(&batch, "dag")?;
        let tid = get_string_column(&batch, "tid")?;
        let taxa = get_string_column(&batch, "taxa")?;
        let antal_platser = get_u64_column(&batch, "antal_platser")?;
        let typ_av_parkering = get_string_column(&batch, "typ_av_parkering")?;
        for i in 0..batch.num_rows() {
            let entry = LocalData {
                valid: get_boolean_with_default(valid, i, false),
                active: get_boolean_with_default(active, i, false),
                postnummer: get_optional_string(postnummer, i),
                adress: get_required_string(address, i),
                gata: get_optional_string(gata, i),
                gatunummer: get_optional_string(gatunummer, i),
                info: get_optional_string(info, i),
                tid: get_optional_string(tid, i),
                dag: get_optional_u8(dag, i),
                taxa: get_optional_string(taxa, i),
                antal_platser: get_optional_u64(antal_platser, i),
                typ_av_parkering: get_optional_string(typ_av_parkering, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Read [`LocalData`] from embedded bytes (Android).
///
/// Memory-based version of [`read_local_parquet`] for reading from
/// embedded assets or byte arrays. Used in Android where data is
/// bundled with the APK or stored in internal storage.
///
/// # Arguments
///
/// * `bytes` - Byte slice containing the parquet file data
///
/// # Returns
///
/// Vector of [`LocalData`] entries.
///
/// # Errors
///
/// Returns error if data is not valid Parquet or schema doesn't match.
///
/// # Examples
///
/// ```no_run
/// use amp_core::parquet::read_local_parquet_from_bytes;
///
/// const EMBEDDED_DATA: &[u8] = include_bytes!("local.parquet");
/// let data = read_local_parquet_from_bytes(EMBEDDED_DATA)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// [`LocalData`]: crate::structs::LocalData
pub fn read_local_parquet_from_bytes(bytes: &[u8]) -> anyhow::Result<Vec<LocalData>> {
    let bytes_obj = Bytes::copy_from_slice(bytes);
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes_obj)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
        let valid = get_boolean_column(&batch, "valid")?;
        let active = get_boolean_column(&batch, "active")?;
        let postnummer = get_string_column(&batch, "postnummer")?;
        let address = get_string_column(&batch, "adress")?;
        let gata = get_string_column(&batch, "gata")?;
        let gatunummer = get_string_column(&batch, "gatunummer")?;
        let info = get_string_column(&batch, "info")?;
        let dag = get_u8_column(&batch, "dag")?;
        let tid = get_string_column(&batch, "tid")?;
        let taxa = get_string_column(&batch, "taxa")?;
        let antal_platser = get_u64_column(&batch, "antal_platser")?;
        let typ_av_parkering = get_string_column(&batch, "typ_av_parkering")?;
        for i in 0..batch.num_rows() {
            let entry = LocalData {
                valid: get_boolean_with_default(valid, i, false),
                active: get_boolean_with_default(active, i, false),
                postnummer: get_optional_string(postnummer, i),
                adress: get_required_string(address, i),
                gata: get_optional_string(gata, i),
                gatunummer: get_optional_string(gatunummer, i),
                info: get_optional_string(info, i),
                tid: get_optional_string(tid, i),
                dag: get_optional_u8(dag, i),
                taxa: get_optional_string(taxa, i),
                antal_platser: get_optional_u64(antal_platser, i),
                typ_av_parkering: get_optional_string(typ_av_parkering, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Read [`AdressClean`] from a parquet file.
///
/// Loads address data with coordinates, typically from the processed
/// address GeoJSON data.
///
/// # Arguments
///
/// * `file` - Open file handle to parquet file
///
/// # Returns
///
/// Vector of [`AdressClean`] entries with Float64 coordinates converted
/// back to [`Decimal`].
///
/// # Errors
///
/// Returns error if file is not valid Parquet or schema doesn't match.
///
/// [`AdressClean`]: crate::structs::AdressClean
/// [`Decimal`]: rust_decimal::Decimal
pub fn read_address_parquet(file: File) -> anyhow::Result<Vec<AdressClean>> {
    let mut reader = create_parquet_reader(file)?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let longitude = batch
            .column(batch.schema().index_of("longitude")?)
            .as_any()
            .downcast_ref::<arrow::array::Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("longitude column missing or wrong type"))?;
        let latitude = batch
            .column(batch.schema().index_of("latitude")?)
            .as_any()
            .downcast_ref::<arrow::array::Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("latitude column missing or wrong type"))?;
        let postnummer = get_string_column(&batch, "postnummer")?;
        let adress = get_string_column(&batch, "adress")?;
        let gata = get_string_column(&batch, "gata")?;
        let gatunummer = get_string_column(&batch, "gatunummer")?;
        for i in 0..batch.num_rows() {
            let entry = AdressClean {
                coordinates: [
                    rust_decimal::Decimal::from_f64(longitude.value(i)).unwrap_or_default(),
                    rust_decimal::Decimal::from_f64(latitude.value(i)).unwrap_or_default(),
                ],
                postnummer: get_optional_string(postnummer, i),
                adress: get_required_string(adress, i),
                gata: get_required_string(gata, i),
                gatunummer: get_required_string(gatunummer, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Write [`OutputData`] to a parquet file.
///
/// Serializes correlated address and parking information for persistent storage.
/// Typically used after running correlation algorithms on the full dataset.
///
/// # Arguments
///
/// * `data` - Vector of [`OutputData`] entries to write
/// * `path` - Output file path
///
/// # Errors
///
/// Returns error if:
/// - `data` is empty (no data to write)
/// - File cannot be created
/// - Parquet write operation fails
///
/// [`OutputData`]: crate::structs::OutputData
pub fn write_output_parquet(data: Vec<OutputData>, path: &str) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty output data"));
    }
    let schema = output_data_schema();
    let writer = create_arrow_writer(path, schema.clone())?;
    let mut postnummer_builder = StringBuilder::new();
    let mut adress_builder = StringBuilder::new();
    let mut gata_builder = StringBuilder::new();
    let mut gatunummer_builder = StringBuilder::new();
    let mut info_builder = StringBuilder::new();
    let mut tid_builder = StringBuilder::new();
    let mut dag_builder = UInt8Builder::new();
    let mut taxa_builder = StringBuilder::new();
    let mut antal_platser_builder = UInt64Builder::new();
    let mut typ_av_parkering_builder = StringBuilder::new();
    for row in data {
        append_optional_string(&mut postnummer_builder, &row.postnummer);
        adress_builder.append_value(&row.adress);
        gata_builder.append_value(&row.gata);
        gatunummer_builder.append_value(&row.gatunummer);
        append_optional_string(&mut info_builder, &row.info);
        append_optional_string(&mut tid_builder, &row.tid);
        append_optional_u8(&mut dag_builder, &row.dag);
        append_optional_string(&mut taxa_builder, &row.taxa);
        append_optional_u64(&mut antal_platser_builder, &row.antal_platser);
        append_optional_string(&mut typ_av_parkering_builder, &row.typ_av_parkering);
    }
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(postnummer_builder.finish()),
            Arc::new(adress_builder.finish()),
            Arc::new(gata_builder.finish()),
            Arc::new(gatunummer_builder.finish()),
            Arc::new(info_builder.finish()),
            Arc::new(tid_builder.finish()),
            Arc::new(dag_builder.finish()),
            Arc::new(taxa_builder.finish()),
            Arc::new(antal_platser_builder.finish()),
            Arc::new(typ_av_parkering_builder.finish()),
        ],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
    write_batch_and_close(writer, batch)
}
/// Write [`AdressClean`] to a parquet file.
///
/// Serializes address data with coordinates for persistent storage.
/// Coordinates are converted from [`Decimal`] to Float64 for compatibility.
///
/// # Arguments
///
/// * `data` - Vector of [`AdressClean`] entries to write
/// * `path` - Output file path
///
/// # Errors
///
/// Returns error if:
/// - `data` is empty
/// - File cannot be created
/// - Parquet write operation fails
///
/// [`AdressClean`]: crate::structs::AdressClean
/// [`Decimal`]: rust_decimal::Decimal
pub fn write_adress_clean_parquet(data: Vec<AdressClean>, path: &str) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty address data"));
    }
    let schema = adress_clean_schema();
    let writer = create_arrow_writer(path, schema.clone())?;
    let mut longitude_builder = Float64Builder::new();
    let mut latitude_builder = Float64Builder::new();
    let mut postnummer_builder = StringBuilder::new();
    let mut adress_builder = StringBuilder::new();
    let mut gata_builder = StringBuilder::new();
    let mut gatunummer_builder = StringBuilder::new();
    for row in data {
        longitude_builder
            .append_value(row.coordinates[0].to_string().parse::<f64>().unwrap_or(0.0));
        latitude_builder.append_value(row.coordinates[1].to_string().parse::<f64>().unwrap_or(0.0));
        append_optional_string(&mut postnummer_builder, &row.postnummer);
        adress_builder.append_value(&row.adress);
        gata_builder.append_value(&row.gata);
        gatunummer_builder.append_value(&row.gatunummer);
    }
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(longitude_builder.finish()),
            Arc::new(latitude_builder.finish()),
            Arc::new(postnummer_builder.finish()),
            Arc::new(adress_builder.finish()),
            Arc::new(gata_builder.finish()),
            Arc::new(gatunummer_builder.finish()),
        ],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
    write_batch_and_close(writer, batch)
}
/// Build [`LocalData`] into an in-memory Parquet buffer.
///
/// Serializes user's saved addresses to a byte vector suitable for:
/// - Passing via JNI to Android
/// - Storing in Android internal storage
/// - Network transmission
///
/// # Arguments
///
/// * `data` - Vector of [`LocalData`] entries to serialize
///
/// # Returns
///
/// Byte vector containing complete Parquet file.
///
/// # Errors
///
/// Returns error if:
/// - `data` is empty
/// - Parquet serialization fails
///
/// # Examples
///
/// ```no_run
/// use amp_core::parquet::build_local_parquet;
/// use amp_core::structs::LocalData;
///
/// let data = vec![/* LocalData entries */];
/// # let data: Vec<LocalData> = vec![];
/// let parquet_bytes = build_local_parquet(data)?;
/// // Write to Android internal storage
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// [`LocalData`]: crate::structs::LocalData
pub fn build_local_parquet(data: Vec<LocalData>) -> anyhow::Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty local data"));
    }
    let schema = local_data_schema();
    let mut buffer = Vec::new();
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(&mut buffer, schema.clone(), Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))?;
    let mut valid_builder = BooleanBuilder::new();
    let mut active_builder = BooleanBuilder::new();
    let mut postnummer_builder = StringBuilder::new();
    let mut adress_builder = StringBuilder::new();
    let mut gata_builder = StringBuilder::new();
    let mut gatunummer_builder = StringBuilder::new();
    let mut info_builder = StringBuilder::new();
    let mut tid_builder = StringBuilder::new();
    let mut dag_builder = UInt8Builder::new();
    let mut taxa_builder = StringBuilder::new();
    let mut antal_platser_builder = UInt64Builder::new();
    let mut typ_av_parkering_builder = StringBuilder::new();
    for row in data {
        valid_builder.append_value(row.valid);
        active_builder.append_value(row.active);
        append_optional_string(&mut postnummer_builder, &row.postnummer);
        adress_builder.append_value(&row.adress);
        append_optional_string(&mut gata_builder, &row.gata);
        append_optional_string(&mut gatunummer_builder, &row.gatunummer);
        append_optional_string(&mut info_builder, &row.info);
        append_optional_string(&mut tid_builder, &row.tid);
        append_optional_u8(&mut dag_builder, &row.dag);
        append_optional_string(&mut taxa_builder, &row.taxa);
        append_optional_u64(&mut antal_platser_builder, &row.antal_platser);
        append_optional_string(&mut typ_av_parkering_builder, &row.typ_av_parkering);
    }
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(valid_builder.finish()),
            Arc::new(active_builder.finish()),
            Arc::new(postnummer_builder.finish()),
            Arc::new(adress_builder.finish()),
            Arc::new(gata_builder.finish()),
            Arc::new(gatunummer_builder.finish()),
            Arc::new(info_builder.finish()),
            Arc::new(tid_builder.finish()),
            Arc::new(dag_builder.finish()),
            Arc::new(taxa_builder.finish()),
            Arc::new(antal_platser_builder.finish()),
            Arc::new(typ_av_parkering_builder.finish()),
        ],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
    writer
        .write(&batch)
        .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(buffer)
}
/// Schema for [`SettingsData`] parquet format.
///
/// Defines 5 non-nullable columns:
/// - `stadning_nu`, `sex_timmar`, `en_dag`: Boolean notification preferences
/// - `theme`: Utf8 ("Light" or "Dark")
/// - `language`: Utf8 ("Svenska", "English", "Espanol", "Francais")
///
/// [`SettingsData`]: crate::structs::SettingsData
pub fn settings_data_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("stadning_nu", DataType::Boolean, false),
        Field::new("sex_timmar", DataType::Boolean, false),
        Field::new("en_dag", DataType::Boolean, false),
        Field::new("theme", DataType::Utf8, false),
        Field::new("language", DataType::Utf8, false),
    ]))
}
/// Build [`SettingsData`] into an in-memory Parquet buffer.
///
/// Serializes user preferences to a byte vector for Android storage.
/// Typically contains only one row (current settings).
///
/// # Arguments
///
/// * `data` - Vector of [`SettingsData`] entries (usually just one)
///
/// # Returns
///
/// Byte vector containing complete Parquet file.
///
/// # Errors
///
/// Returns error if:
/// - `data` is empty
/// - Parquet serialization fails
///
/// [`SettingsData`]: crate::structs::SettingsData
pub fn build_settings_parquet(data: Vec<SettingsData>) -> anyhow::Result<Vec<u8>> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty settings data"));
    }
    let schema = settings_data_schema();
    let mut buffer = Vec::new();
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(&mut buffer, schema.clone(), Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))?;
    let mut stadning_nu_builder = BooleanBuilder::new();
    let mut sex_timmar_builder = BooleanBuilder::new();
    let mut en_dag_builder = BooleanBuilder::new();
    let mut theme_builder = StringBuilder::new();
    let mut language_builder = StringBuilder::new();
    for row in data {
        stadning_nu_builder.append_value(row.stadning_nu);
        sex_timmar_builder.append_value(row.sex_timmar);
        en_dag_builder.append_value(row.en_dag);
        theme_builder.append_value(&row.theme);
        language_builder.append_value(&row.language);
    }
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(stadning_nu_builder.finish()),
            Arc::new(sex_timmar_builder.finish()),
            Arc::new(en_dag_builder.finish()),
            Arc::new(theme_builder.finish()),
            Arc::new(language_builder.finish()),
        ],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
    writer
        .write(&batch)
        .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(buffer)
}
/// Read [`SettingsData`] from a parquet file.
///
/// Loads user preferences from persistent storage.
///
/// # Arguments
///
/// * `file` - Open file handle to parquet file
///
/// # Returns
///
/// Vector of [`SettingsData`] entries (typically just one row).
///
/// # Errors
///
/// Returns error if file is not valid Parquet or schema doesn't match.
///
/// [`SettingsData`]: crate::structs::SettingsData
pub fn read_settings_parquet(file: File) -> anyhow::Result<Vec<SettingsData>> {
    let mut reader = create_parquet_reader(file)?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let stadning_nu = get_boolean_column(&batch, "stadning_nu")?;
        let sex_timmar = get_boolean_column(&batch, "sex_timmar")?;
        let en_dag = get_boolean_column(&batch, "en_dag")?;
        let theme = get_string_column(&batch, "theme")?;
        let language = get_string_column(&batch, "language")?;
        for i in 0..batch.num_rows() {
            let entry = SettingsData {
                stadning_nu: get_boolean_with_default(stadning_nu, i, true),
                sex_timmar: get_boolean_with_default(sex_timmar, i, true),
                en_dag: get_boolean_with_default(en_dag, i, false),
                theme: get_required_string(theme, i),
                language: get_required_string(language, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Read [`SettingsData`] from embedded bytes (Android).
///
/// Memory-based version of [`read_settings_parquet`] for reading from
/// embedded assets or byte arrays.
///
/// # Arguments
///
/// * `bytes` - Byte slice containing the parquet file data
///
/// # Returns
///
/// Vector of [`SettingsData`] entries.
///
/// # Errors
///
/// Returns error if data is not valid Parquet or schema doesn't match.
///
/// [`SettingsData`]: crate::structs::SettingsData
pub fn read_settings_parquet_from_bytes(bytes: &[u8]) -> anyhow::Result<Vec<SettingsData>> {
    let bytes_obj = Bytes::copy_from_slice(bytes);
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes_obj)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    for batch_result in reader {
        let batch = batch_result.map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
        let stadning_nu = get_boolean_column(&batch, "stadning_nu")?;
        let sex_timmar = get_boolean_column(&batch, "sex_timmar")?;
        let en_dag = get_boolean_column(&batch, "en_dag")?;
        let theme = get_string_column(&batch, "theme")?;
        let language = get_string_column(&batch, "language")?;
        for i in 0..batch.num_rows() {
            let entry = SettingsData {
                stadning_nu: get_boolean_with_default(stadning_nu, i, true),
                sex_timmar: get_boolean_with_default(sex_timmar, i, true),
                en_dag: get_boolean_with_default(en_dag, i, false),
                theme: get_required_string(theme, i),
                language: get_required_string(language, i),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
