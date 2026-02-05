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
/// Extract a StringArray column from a RecordBatch
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
/// Extract a BooleanArray column from a RecordBatch
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
/// Extract a UInt8Array column from a RecordBatch
fn get_u8_column<'a>(batch: &'a RecordBatch, column_name: &str) -> anyhow::Result<&'a UInt8Array> {
    batch
        .column(batch.schema().index_of(column_name)?)
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| anyhow::anyhow!("{} column missing or wrong type", column_name))
}
/// Extract a UInt64Array column from a RecordBatch
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
/// Get optional string value from StringArray at index
fn get_optional_string(array: &StringArray, index: usize) -> Option<String> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index).to_string())
    }
}
/// Get required string value from StringArray at index (returns empty string if null)
fn get_required_string(array: &StringArray, index: usize) -> String {
    if array.is_null(index) {
        String::new()
    } else {
        array.value(index).to_string()
    }
}
/// Get optional u8 value from UInt8Array at index
fn get_optional_u8(array: &UInt8Array, index: usize) -> Option<u8> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index))
    }
}
/// Get optional u64 value from UInt64Array at index
fn get_optional_u64(array: &UInt64Array, index: usize) -> Option<u64> {
    if array.is_null(index) {
        None
    } else {
        Some(array.value(index))
    }
}
/// Get boolean value from BooleanArray at index (defaults to false if null)
fn get_boolean_with_default(array: &BooleanArray, index: usize, default: bool) -> bool {
    if array.is_null(index) {
        default
    } else {
        array.value(index)
    }
}
/// Create a Parquet reader from a file
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
/// Append optional string to StringBuilder
fn append_optional_string(builder: &mut StringBuilder, value: &Option<String>) {
    match value {
        Some(v) => builder.append_value(v.clone()),
        None => builder.append_null(),
    }
}
/// Append optional u8 to UInt8Builder
fn append_optional_u8(builder: &mut UInt8Builder, value: &Option<u8>) {
    match value {
        Some(v) => builder.append_value(*v),
        None => builder.append_null(),
    }
}
/// Append optional u64 to UInt64Builder
fn append_optional_u64(builder: &mut UInt64Builder, value: &Option<u64>) {
    match value {
        Some(v) => builder.append_value(*v),
        None => builder.append_null(),
    }
}
/// Create ArrowWriter with standard properties
fn create_arrow_writer(path: &str, schema: Arc<Schema>) -> anyhow::Result<ArrowWriter<File>> {
    let file = File::create(path).map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))
}
/// Write a single batch and close the writer
fn write_batch_and_close(mut writer: ArrowWriter<File>, batch: RecordBatch) -> anyhow::Result<()> {
    writer
        .write(&batch)
        .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}

/// Load debug addresses from minimal parquet file (only contains address strings)
/// 
/// This function reads a parquet file with ONLY the `adress` field populated.
/// All other fields are NULL. This mimics the user clicking "Add Address" button
/// multiple times with different addresses.
/// 
/// The returned `StoredAddress` objects can then be matched against the static
/// parking database using fuzzy matching via `StoredAddress::to_local_data()`.
/// 
/// # Arguments
/// * `bytes` - Byte slice containing the minimal debug.parquet file data
/// 
/// # Returns
/// Vector of StoredAddress entries, each containing only an address string
/// 
/// # Example
/// ```no_run
/// use amp_core::parquet::load_debug_addresses;
/// 
/// const DEBUG_DATA: &[u8] = include_bytes!("../../../android/app/src/main/assets/debug.parquet");
/// let addresses = load_debug_addresses(DEBUG_DATA).unwrap();
/// println!("Loaded {} debug addresses", addresses.len());
/// ```
pub fn load_debug_addresses(bytes: &[u8]) -> anyhow::Result<Vec<StoredAddress>> {
    println!("[load_debug_addresses] Loading debug addresses from {} bytes", bytes.len());
    
    let bytes_obj = Bytes::copy_from_slice(bytes);
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes_obj)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    
    let reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    
    let mut result = Vec::new();
    
    for batch_result in reader {
        let batch = batch_result.map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
        
        // Get the address column (only column we care about)
        let address_array = get_string_column(&batch, "adress")?;
        
        for i in 0..batch.num_rows() {
            let address_str = get_required_string(address_array, i);
            if !address_str.is_empty() {
                println!("[load_debug_addresses] Loaded address: '{}'", address_str);
                result.push(StoredAddress::new(address_str));
            }
        }
    }
    
    println!("[load_debug_addresses] Successfully loaded {} debug addresses", result.len());
    Ok(result)
}

/// Load debug addresses from a file path (for non-Android/desktop testing)
/// 
/// # Arguments
/// * `path` - Path to the debug.parquet file
/// 
/// # Returns
/// Vector of StoredAddress entries
pub fn load_debug_addresses_from_file(path: &str) -> anyhow::Result<Vec<StoredAddress>> {
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
        
        for i in 0..batch.num_rows() {
            let address_str = get_required_string(address_array, i);
            if !address_str.is_empty() {
                result.push(StoredAddress::new(address_str));
            }
        }
    }
    
    Ok(result)
}

/// Read parking data from parquet file
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
/// Read stored data from parquet file
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
/// Read LocalData from embedded bytes (for Android debug mode)
///
/// This function is similar to read_local_parquet but takes a byte slice
/// instead of a File, making it suitable for reading from embedded assets.
///
/// # Arguments
/// * `bytes` - Byte slice containing the parquet file data
///
/// # Returns
/// Vector of LocalData entries
///
/// # Example
/// ```no_run
/// use amp_core::parquet::read_local_parquet_from_bytes;
///
/// const DEBUG_DATA: &[u8] = include_bytes!("debug.parquet");
/// let data = read_local_parquet_from_bytes(DEBUG_DATA).unwrap();
/// ```
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
/// Read address clean data from parquet file
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
/// Write OutputData to parquet file
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
/// Write AdressClean to parquet file
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
/// Build LocalData into a parquet-encoded in-memory buffer
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
