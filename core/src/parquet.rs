use crate::structs::*;
use anyhow;
use arrow::{
    array::{BooleanArray, BooleanBuilder, StringArray, StringBuilder, UInt8Array, UInt8Builder},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::{
    arrow::ArrowWriter,
    arrow::arrow_reader::ParquetRecordBatchReaderBuilder,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::{collections::BTreeMap, fs::File, sync::Arc};

pub fn read_db_parquet() -> anyhow::Result<Vec<AdressInfo>> {
    let file = File::open("adress_info.parquet").expect("Failed to open adress_info.parquet");

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        let relevant = batch
            .column(batch.schema().index_of("relevant")?)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("relevant column missing or wrong type");

        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("postnummer column missing or wrong type");

        let adress = batch
            .column(batch.schema().index_of("adress")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("adress column missing or wrong type");

        let gata = batch
            .column(batch.schema().index_of("gata")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gata column missing or wrong type");

        let gatunummer = batch
            .column(batch.schema().index_of("gatunummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gatunummer column missing or wrong type");

        let info = batch
            .column(batch.schema().index_of("info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("info column missing or wrong type");

        let tid = batch
            .column(batch.schema().index_of("tid")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("tid column missing or wrong type");

        let dag = batch
            .column(batch.schema().index_of("dag")?)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("dag column missing or wrong type");

        // Convert rows to AdressInfo
        for i in 0..batch.num_rows() {
            let entry = AdressInfo {
                relevant: relevant.value(i),
                postnummer: postnummer.value(i).to_string(),
                adress: adress.value(i).to_string(),
                gata: gata.value(i).to_string(),
                gatunummer: gatunummer.value(i).to_string(),
                info: info.value(i).to_string(),
                tid: tid.value(i).to_string(),
                dag: dag.value(i),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

pub fn read_local_parquet() -> anyhow::Result<Vec<Local>> {
    let file = File::open("local.parquet").expect("Failed to open local.parquet");

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("postnummer column missing or wrong type");

        let adress = batch
            .column(batch.schema().index_of("adress")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("adress column missing or wrong type");

        let gata = batch
            .column(batch.schema().index_of("gata")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gata column missing or wrong type");

        let gatunummer = batch
            .column(batch.schema().index_of("gatunummer")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("gatunummer column missing or wrong type");

        let info = batch
            .column(batch.schema().index_of("info")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("info column missing or wrong type");

        let tid = batch
            .column(batch.schema().index_of("tid")?)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("tid column missing or wrong type");

        let dag = batch
            .column(batch.schema().index_of("dag")?)
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("dag column missing or wrong type");

        let active = batch
            .column(batch.schema().index_of("active")?)
            .as_any()
            .downcast_ref::<BooleanArray>()
            .expect("active column missing or wrong type");

        // Convert rows to AdressInfo
        for i in 0..batch.num_rows() {
            let entry = Local {
                postnummer: postnummer.value(i).to_string(),
                adress: adress.value(i).to_string(),
                gata: gata.value(i).to_string(),
                gatunummer: gatunummer.value(i).to_string(),
                info: info.value(i).to_string(),
                tid: tid.value(i).to_string(),
                dag: dag.value(i),
                active: active.value(i),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

pub fn write_local_parquet(data: Vec<Local>) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err::<(), anyhow::Error>(anyhow::anyhow!("Empty local parquet"));
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::Utf8, false),
        Field::new("gata", DataType::Utf8, false),
        Field::new("adress", DataType::Utf8, false),
        Field::new("gatunummer", DataType::Utf8, false),
        Field::new("dag", DataType::UInt8, false),
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("active", DataType::Boolean, false),
    ]));

    let mut grouped: BTreeMap<String, Vec<Local>> = BTreeMap::new();

    for d in data {
        let key = d.postnummer.clone();
        grouped.entry(key).or_insert_with(Vec::new).push(d);
    }

    let path = "local.parquet".to_string();
    let file = File::create(&path).expect("Failed to create file");
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .expect("Failed to create ArrowWriter");

    for (postnummer, rows) in grouped {
        let mut post_builder = StringBuilder::new();
        let mut gata_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut nr_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut active_builder = BooleanBuilder::new();

        for r in rows {
            post_builder.append_value(&postnummer);
            gata_builder.append_value(&r.gata);
            adress_builder.append_value(&r.adress);
            nr_builder.append_value(&r.gatunummer);
            dag_builder.append_value(r.dag);
            tid_builder.append_value(&r.tid);
            info_builder.append_value(&r.info);
            active_builder.append_value(r.active);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(post_builder.finish()),
                Arc::new(gata_builder.finish()),
                Arc::new(adress_builder.finish()),
                Arc::new(nr_builder.finish()),
                Arc::new(dag_builder.finish()),
                Arc::new(tid_builder.finish()),
                Arc::new(info_builder.finish()),
                Arc::new(active_builder.finish()),
            ],
        )
        .expect("Failed to create record batch");

        writer.write(&batch).expect("Failed to write batch");
    }

    writer.close().expect("Writer failed to close");

    Ok(())
}

pub fn write_correlation(data: Vec<AdressInfo>) -> anyhow::Result<()> {
    if data.is_empty() {
        return Err::<(), anyhow::Error>(anyhow::anyhow!("Empty info parquet"));
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::Utf8, false),
        Field::new("gata", DataType::Utf8, false),
        Field::new("adress", DataType::Utf8, false),
        Field::new("gatunummer", DataType::Utf8, false),
        Field::new("dag", DataType::UInt8, false),
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("relevant", DataType::Boolean, false),
    ]));

    let mut grouped: BTreeMap<String, Vec<AdressInfo>> = BTreeMap::new();

    for d in data {
        let key = d.postnummer.clone();
        grouped.entry(key).or_insert_with(Vec::new).push(d);
    }

    let path = "adress_info.parquet".to_string();
    let file = File::create(&path).expect("Failed to create file");
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))
        .expect("Failed to create ArrowWriter");

    for (postnummer, rows) in grouped {
        let mut post_builder = StringBuilder::new();
        let mut gata_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut nr_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut relevant_builder = BooleanBuilder::new();

        for r in rows {
            post_builder.append_value(&postnummer);
            gata_builder.append_value(&r.gata);
            adress_builder.append_value(&r.adress);
            nr_builder.append_value(&r.gatunummer);
            dag_builder.append_value(r.dag);
            tid_builder.append_value(&r.tid);
            info_builder.append_value(&r.info);
            relevant_builder.append_value(r.relevant);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(post_builder.finish()),
                Arc::new(gata_builder.finish()),
                Arc::new(adress_builder.finish()),
                Arc::new(nr_builder.finish()),
                Arc::new(dag_builder.finish()),
                Arc::new(tid_builder.finish()),
                Arc::new(info_builder.finish()),
                Arc::new(relevant_builder.finish()),
            ],
        )
        .expect("Failed to create record batch");

        writer.write(&batch).expect("Failed to write batch");
    }

    writer.close().expect("Writer failed to close");

    Ok(())
}
