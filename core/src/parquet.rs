use crate::structs::*;
use anyhow;
use arrow::{
    array::{StringArray, StringBuilder, Float64Array, Float64Builder, BooleanArray, BooleanBuilder},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::ArrowWriter,
    arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::{collections::BTreeMap, fs::File, sync::Arc};

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

        // Convert rows to CorrelationResult
        for i in 0..batch.num_rows() {
            let miljo_match = if let Some(Some(dist)) = miljo_dist.clone().nth(i) {
                Some((dist, miljo_info.clone().nth(i).flatten().map(|s| s.to_string()).unwrap_or_default()))
            } else {
                None
            };

            let parkering_match = if let Some(Some(dist)) = parkering_dist.clone().nth(i) {
                Some((dist, parkering_info.clone().nth(i).flatten().map(|s| s.to_string()).unwrap_or_default()))
            } else {
                None
            };

            let entry = CorrelationResult {
                address: address.clone().nth(i).flatten().map(|s| s.to_string()).unwrap_or_default(),
                postnummer: postnummer.clone().nth(i).flatten().map(|s| s.to_string()).unwrap_or_default(),
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
        grouped.entry(key).or_insert_with(Vec::new).push(result);
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

        writer.write(&batch)
            .map_err(|e| anyhow::anyhow!("Failed to write batch: {}", e))?;
    }

    writer.close()
        .map_err(|e| anyhow::anyhow!("Failed to close writer: {}", e))?;

    Ok(())
}
