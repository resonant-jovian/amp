use crate::structs::*;
use anyhow;
use arrow::array::UInt16Builder;
use arrow::{
    array::{
        Float64Array, Float64Builder, StringArray, StringBuilder, UInt8Array,
        UInt8Builder,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::ArrowWriter, arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::{collections::BTreeMap, fs::File, sync::Arc};
/// Parking restriction info extracted from parquet data
#[derive(Clone, Debug, PartialEq)]
pub struct ParkingRestriction {
    pub gata: String,
    pub gatunummer: String,
    pub postnummer: u16,
    pub adress: String,
    pub dag: u8,
    pub tid: String,
    pub info: String,
}
/// Read correlation results from parquet file
pub fn read_correlation_parquet() -> anyhow::Result<Vec<CorrelationResult>> {
    let file = File::open("correlation_results.parquet")
        .map_err(|e| {
            anyhow::anyhow!("Failed to open correlation_results.parquet: {}", e)
        })?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let mut reader = builder
        .build()
        .map_err(|e| {
            anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e)
        })?;
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
            .ok_or_else(|| {
                anyhow::anyhow!("miljo_distance column missing or wrong type")
            })?
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
            .ok_or_else(|| {
                anyhow::anyhow!("parkering_distance column missing or wrong type")
            })?
            .iter();
        let parkering_info = batch
            .column(batch.schema().index_of("parkering_info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                anyhow::anyhow!("parkering_info column missing or wrong type")
            })?
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
            let parkering_match = if let Some(Some(dist)) = parkering_dist.clone().nth(i)
            {
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
    let schema = Arc::new(
        Schema::new(
            vec![
                Field::new("address", DataType::Utf8, false),
                Field::new("postnummer", DataType::Utf8, false),
                Field::new("miljo_distance", DataType::Float64, true),
                Field::new("miljo_info", DataType::Utf8, true),
                Field::new("parkering_distance", DataType::Float64, true),
                Field::new("parkering_info", DataType::Utf8, true),
            ],
        ),
    );
    let mut grouped: BTreeMap<String, Vec<CorrelationResult>> = BTreeMap::new();
    for result in data {
        let key = result.postnummer.clone();
        grouped.entry(key).or_default().push(result);
    }
    let path = "correlation_results.parquet";
    let file = File::create(path)
        .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
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
    writer.close().map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}
/// Schema for Android local address storage (parquet format)
pub fn android_local_schema() -> Arc<Schema> {
    Arc::new(
        Schema::new(
            vec![
                Field::new("gata", DataType::Utf8, false),
                Field::new("gatunummer", DataType::Utf8, false),
                Field::new("postnummer", DataType::UInt16, false),
                Field::new("adress", DataType::Utf8, false),
                Field::new("dag", DataType::UInt8, false),
                Field::new("tid", DataType::Utf8, false),
                Field::new("info", DataType::Utf8, false),
                Field::new("distance", DataType::Float64, false),
            ],
        ),
    )
}
/// Read Android local addresses from parquet file
pub fn read_android_local_addresses(
    path: &str,
) -> anyhow::Result<Vec<ParkingRestriction>> {
    let file = File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open {}: {}", path, e))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| anyhow::anyhow!("Failed to create Parquet reader builder: {}", e))?;
    let mut reader = builder
        .build()
        .map_err(|e| {
            anyhow::anyhow!("Failed to build Parquet record batch reader: {}", e)
        })?;
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
                gata: gata
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                gatunummer: gatunummer
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                postnummer: postnummer.clone().nth(i).flatten().unwrap_or(0),
                adress: adress
                    .clone()
                    .nth(i)
                    .flatten()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                dag: dag.clone().nth(i).flatten().unwrap_or(0),
                tid: tid
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
        grouped.entry(addr.postnummer).or_default().push(addr);
    }
    let file = File::create(path)
        .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", path, e))?;
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
            gata_builder.append_value(&addr.gata);
            gatunummer_builder.append_value(&addr.gatunummer);
            postnummer_builder.append_value(addr.postnummer);
            adress_builder.append_value(&addr.adress);
            dag_builder.append_value(addr.dag);
            tid_builder.append_value(&addr.tid);
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
    writer.close().map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;
    Ok(())
}
