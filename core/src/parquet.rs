use crate::structs::*;
use anyhow;
use arrow::array::UInt16Builder;
use arrow::{
    array::{Float64Array, Float64Builder, StringArray, StringBuilder, UInt8Array, UInt8Builder},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::ArrowWriter,
    arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::{collections::BTreeMap, fs::File, sync::Arc};
/// Parking restriction information extracted from parquet data
///
/// Represents a single parking restriction entry with address details,
/// day of week, time period, and additional information.
#[derive(Clone, Debug, PartialEq)]
pub struct ParkingRestriction {
    /// Street name (e.g., "Storgatan")
    pub street: String,
    /// Street number (e.g., "10")
    pub street_number: String,
    /// Postal code (e.g., 22100)
    pub postal_code: u16,
    /// Full address string
    pub address: String,
    /// Day of week (0-6, where 0 is Monday)
    pub day: u8,
    /// Time period (e.g., "08:00-12:00")
    pub time: String,
    /// Additional information about the restriction
    pub info: String,
}

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

/// Read correlation results from parquet file
pub fn read_correlation_parquet() -> anyhow::Result<Vec<CorrelationResult>> {
    let file = File::open("correlation_results.parquet")
        .map_err(|e| anyhow::anyhow!("Failed to open correlation_results.parquet: {}", e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let mut reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;
        let address = batch
            .column(batch.schema().index_of("address")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("address column missing or wrong type"))?
            .iter();
        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("postnummer column missing or wrong type"))?
            .iter();
        let miljo_dist = batch
            .column(batch.schema().index_of("miljo_distance")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("miljo_distance column missing or wrong type"))?
            .iter();
        let miljo_info = batch
            .column(batch.schema().index_of("miljo_info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("miljo_info column missing or wrong type"))?
            .iter();
        let parkering_dist = batch
            .column(batch.schema().index_of("parkering_distance")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("parkering_distance column missing or wrong type"))?
            .iter();
        let parkering_info = batch
            .column(batch.schema().index_of("parkering_info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("parkering_info column missing or wrong type"))?
            .iter();
        for i in 0..batch.num_rows() {
            let miljo_match = if let Some(Some(dist)) = miljo_dist.clone().nth(i) {
                Some((
                    dist,
                    miljo_info
                        .clone()
                        .nth(i)
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                ))
            } else {
                None
            };
            let parkering_match = if let Some(Some(dist)) = parkering_dist.clone().nth(i) {
                Some((
                    dist,
                    parkering_info
                        .clone()
                        .nth(i)
                        .flatten()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                ))
            } else {
                None
            };
            let entry = CorrelationResult {
                address: address
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                postnummer: postnummer
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                miljo_match,
                parkering_match,
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Write correlation results to parquet file
pub fn write_correlation_parquet(data: Vec<CorrelationResult>) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty correlation results"));
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("address", DataType::Utf8, false),
        Field::new("postnummer", DataType::Utf8, false),
        Field::new("miljo_distance", DataType::Float64, true),
        Field::new("miljo_info", DataType::Utf8, true),
        Field::new("parkering_distance", DataType::Float64, true),
        Field::new("parkering_info", DataType::Utf8, true),
    ]));
    let mut grouped: BTreeMap<String, Vec<CorrelationResult>> = BTreeMap::new();
    for result in data {
        let key = result.postnummer.clone();
        grouped.entry(key).or_default().push(result);
    }
    let path = "correlation_results.parquet";
    let file = File::create(path).map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))?;
    for (_, rows) in grouped {
        let mut address_builder = StringBuilder::new();
        let mut postnummer_builder = StringBuilder::new();
        let mut miljo_dist_builder = Float64Builder::new();
        let mut miljo_info_builder = StringBuilder::new();
        let mut parkering_dist_builder = Float64Builder::new();
        let mut parkering_info_builder = StringBuilder::new();
        for r in rows {
            address_builder.append_value(&r.address);
            postnummer_builder.append_value(&r.postnummer);
            match &r.miljo_match {
                Some((dist, info)) => {
                    miljo_dist_builder.append_value(*dist);
                    miljo_info_builder.append_value(info);
                }
                None => {
                    miljo_dist_builder.append_null();
                    miljo_info_builder.append_null();
                }
            }
            match &r.parkering_match {
                Some((dist, info)) => {
                    parkering_dist_builder.append_value(*dist);
                    parkering_info_builder.append_value(info);
                }
                None => {
                    parkering_dist_builder.append_null();
                    parkering_info_builder.append_null();
                }
            }
        }
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(address_builder.finish()),
                Arc::new(postnummer_builder.finish()),
                Arc::new(miljo_dist_builder.finish()),
                Arc::new(miljo_info_builder.finish()),
                Arc::new(parkering_dist_builder.finish()),
                Arc::new(parkering_info_builder.finish()),
            ],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
        writer
            .write(&batch)
            .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    }
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}
/// Schema for Android local address storage (parquet format)
///
/// Defines the structure for persisting parking restriction data.
/// Note: Column names in parquet use Swedish for backwards compatibility
/// with existing data files, but struct fields use English.
pub fn android_local_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("gata", DataType::Utf8, false),
        Field::new("gatunummer", DataType::Utf8, false),
        Field::new("postnummer", DataType::UInt16, false),
        Field::new("adress", DataType::Utf8, false),
        Field::new("dag", DataType::UInt8, false),
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("distance", DataType::Float64, false),
    ]))
}
/// Read Android local addresses from parquet file
///
/// # Arguments
/// * `path` - Path to the parquet file
///
/// # Returns
/// Vector of ParkingRestriction entries with translated field names
pub fn read_android_local_addresses(path: &str) -> anyhow::Result<Vec<ParkingRestriction>> {
    let file = File::open(path).map_err(|e| anyhow::anyhow!("Failed to open {}: {}", path, e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let mut reader = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e))?;
    let mut result = Vec::new();
    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;
        let gata = batch
            .column(batch.schema().index_of("gata")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("gata column missing or wrong type"))?
            .iter();
        let gatunummer = batch
            .column(batch.schema().index_of("gatunummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("gatunummer column missing or wrong type"))?
            .iter();
        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<arrow::array::UInt16Array>()
            .ok_or_else(|| anyhow::anyhow!("postnummer column missing or wrong type"))?
            .iter();
        let adress = batch
            .column(batch.schema().index_of("adress")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("adress column missing or wrong type"))?
            .iter();
        let dag = batch
            .column(batch.schema().index_of("dag")?)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| anyhow::anyhow!("dag column missing or wrong type"))?
            .iter();
        let tid = batch
            .column(batch.schema().index_of("tid")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("tid column missing or wrong type"))?
            .iter();
        let info = batch
            .column(batch.schema().index_of("info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("info column missing or wrong type"))?
            .iter();
        for i in 0..batch.num_rows() {
            let entry = ParkingRestriction {
                street: gata
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                street_number: gatunummer
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                postal_code: postnummer.clone().nth(i).flatten().unwrap_or(0),
                address: adress
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                day: dag.clone().nth(i).flatten().unwrap_or(0),
                time: tid
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                info: info
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            };
            result.push(entry);
        }
    }
    Ok(result)
}
/// Write Android local addresses to parquet file
///
/// # Arguments
/// * `path` - Output path for the parquet file
/// * `addresses` - Vector of ParkingRestriction entries to write
///
/// # Returns
/// Result indicating success or failure
pub fn write_android_local_addresses(
    path: &str,
    addresses: Vec<ParkingRestriction>,
) -> anyhow::Result<()> {
    if addresses.is_empty() {
        return Err(anyhow::anyhow!("Empty address list"));
    }
    let schema = android_local_schema();
    let mut grouped: BTreeMap<u16, Vec<ParkingRestriction>> = BTreeMap::new();
    for addr in addresses {
        grouped.entry(addr.postal_code).or_default().push(addr);
    }
    let file =
        File::create(path).map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", path, e))?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))?;
    for (_, rows) in grouped {
        let mut gata_builder = StringBuilder::new();
        let mut gatunummer_builder = StringBuilder::new();
        let mut postnummer_builder = UInt16Builder::new();
        let mut adress_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut distance_builder = Float64Builder::new();
        for addr in rows {
            gata_builder.append_value(&addr.street);
            gatunummer_builder.append_value(&addr.street_number);
            postnummer_builder.append_value(addr.postal_code);
            adress_builder.append_value(&addr.address);
            dag_builder.append_value(addr.day);
            tid_builder.append_value(&addr.time);
            info_builder.append_value(&addr.info);
            distance_builder.append_value(0.0);
        }
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(gata_builder.finish()),
                Arc::new(gatunummer_builder.finish()),
                Arc::new(postnummer_builder.finish()),
                Arc::new(adress_builder.finish()),
                Arc::new(dag_builder.finish()),
                Arc::new(tid_builder.finish()),
                Arc::new(info_builder.finish()),
                Arc::new(distance_builder.finish()),
            ],
        )
        .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))?;
        writer
            .write(&batch)
            .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    }
    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}
pub fn write_output_data(
    path: &str,
    data: Vec<OutputData>,
) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Empty output data"));
    }

    let schema = output_data_schema();

    // Group by postnummer
    let mut grouped: BTreeMap<Option<String>, Vec<OutputData>> = BTreeMap::new();
    for item in data {
        grouped.entry(item.postnummer.clone()).or_default().push(item);
    }

    let file = File::create(path)
        .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", path, e))?;

    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();

    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .map_err(|e| anyhow::anyhow!("Failed to create ArrowWriter: {}", e))?;

    for (_, rows) in grouped {
        let mut postnummer_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut gata_builder = StringBuilder::new();
        let mut gatunummer_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut tid_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut taxa_builder = StringBuilder::new();
        let mut antal_platser_builder = arrow::array::UInt64Builder::new();
        let mut typ_av_parkering_builder = StringBuilder::new();

        for item in rows {
            // postnummer
            match &item.postnummer {
                Some(v) => postnummer_builder.append_value(v),
                None => postnummer_builder.append_null(),
            }

            // Required fields
            adress_builder.append_value(&item.adress);
            gata_builder.append_value(&item.gata);
            gatunummer_builder.append_value(&item.gatunummer);

            // Optional miljö fields
            match &item.info {
                Some(v) => info_builder.append_value(v),
                None => info_builder.append_null(),
            }
            match &item.tid {
                Some(v) => tid_builder.append_value(v),
                None => tid_builder.append_null(),
            }
            match item.dag {
                Some(v) => dag_builder.append_value(v),
                None => dag_builder.append_null(),
            }

            // Optional parkering fields
            match &item.taxa {
                Some(v) => taxa_builder.append_value(v),
                None => taxa_builder.append_null(),
            }
            match item.antal_platser {
                Some(v) => antal_platser_builder.append_value(v),
                None => antal_platser_builder.append_null(),
            }
            match &item.typ_av_parkering {
                Some(v) => typ_av_parkering_builder.append_value(v),
                None => typ_av_parkering_builder.append_null(),
            }
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

        writer
            .write(&batch)
            .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    }

    writer
        .close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;

    Ok(())
}
