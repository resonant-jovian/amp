use arrow::array::{
    Float64Array, Float64Builder, StringArray, StringBuilder, UInt8Array, UInt8Builder,
    UInt16Array, UInt16Builder, UInt64Builder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::file::properties::{EnabledStatistics, WriterProperties};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::f64;
use std::fs::File;
use std::sync::Arc;

#[derive(Debug)]
pub struct AdressClean {
    pub coordinates: [f64; 2],
    pub postnummer: u16,
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
}

#[derive(Debug)]
pub struct MiljoeDataClean {
    pub coordinates: [[f64; 2]; 2],
    pub info: String,
    pub tid: String,
    pub dag: u8,
}

struct AdressInfo {
    debug_closest_line_id: u64,
    coordinates: [f64; 2],
    postnummer: u16,
    adress: String,
    gata: String,
    gatunummer: String,
    info: String,
    tid: String,
    dag: u8,
}

fn main() {
    let lines = read_miljoeparkering().expect("failed to read file");
    let points = read_adresser().expect("failed to read file");

    let results = find_closest_lines(&points, &lines);

    for (i, res) in results.iter().enumerate() {
        match res {
            Some((line_idx, dist)) => {
                println!(
                    "Point {} is closest to line {} ({}), distance {:.3}",
                    i, line_idx, lines[*line_idx].info, dist
                );
            }
            None => println!("Point {} has no closest line", i),
        }
    }
}

/// Squared distance from point to line segment
fn distance_point_to_line_squared(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let ab_len_sq = ab[0] * ab[0] + ab[1] * ab[1];

    if ab_len_sq == 0.0 {
        return ap[0] * ap[0] + ap[1] * ap[1];
    }

    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / ab_len_sq).clamp(0.0, 1.0);
    let closest = [a[0] + t * ab[0], a[1] + t * ab[1]];
    let dx = p[0] - closest[0];
    let dy = p[1] - closest[1];
    dx * dx + dy * dy
}

/// Find the closest line index + distance for each point in parallel
pub fn find_closest_lines(
    points: &[AdressClean],
    lines: &[MiljoeDataClean],
) -> Vec<Option<(usize, f64)>> {
    points
        .par_iter()
        .map(|point| {
            lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    (
                        i,
                        distance_point_to_line_squared(
                            point.coordinates,
                            line.coordinates[0],
                            line.coordinates[1],
                        ),
                    )
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(i, dist_sq)| (i, dist_sq.sqrt()))
        })
        .collect()
}

pub fn read_adresser() -> anyhow::Result<Vec<AdressClean>> {
    // Open the Parquet file
    let file = File::open("adresser.parquet").expect("Failed to open adresser.parquet");

    // Build a reader that yields Arrow RecordBatches
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        // Look up each column by name and downcast to specific Arrow array type
        let lat = batch
            .column(batch.schema().index_of("lat")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("latitude column missing or wrong type");

        let lon = batch
            .column(batch.schema().index_of("lon")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("longitude column missing or wrong type");

        let postnummer = batch
            .column(batch.schema().index_of("postnummer")?)
            .as_any()
            .downcast_ref::<UInt16Array>()
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

        // Convert each row in the batch into our struct
        for i in 0..batch.num_rows() {
            let entry = AdressClean {
                coordinates: [lat.value(i), lon.value(i)],
                postnummer: postnummer.value(i),
                adress: adress.value(i).to_string(),
                gata: gata.value(i).to_string(),
                gatunummer: gatunummer.value(i).to_string(),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

pub fn read_miljoeparkering() -> anyhow::Result<Vec<MiljoeDataClean>> {
    // Open the Parquet file
    let file = File::open("miljöparkering.parquet").expect("Failed to open miljöparkering.parquet");

    // Build a Parquet reader
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create Parquet reader builder");

    let mut reader = builder
        .build()
        .expect("Failed to build Parquet record batch reader");

    let mut result = Vec::new();

    while let Some(batch) = reader.next().transpose()? {
        let batch: RecordBatch = batch;

        // Downcast each column to the correct type
        let lat_start = batch
            .column(batch.schema().index_of("lat_start")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lat_start column missing or wrong type");

        let lon_start = batch
            .column(batch.schema().index_of("lon_start")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lon_start column missing or wrong type");

        let lat_end = batch
            .column(batch.schema().index_of("lat_end")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lat_end column missing or wrong type");

        let lon_end = batch
            .column(batch.schema().index_of("lon_end")?)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lon_end column missing or wrong type");

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

        // Convert rows to MiljoeDataClean
        for i in 0..batch.num_rows() {
            let entry = MiljoeDataClean {
                coordinates: [
                    [lat_start.value(i), lon_start.value(i)],
                    [lat_end.value(i), lon_end.value(i)],
                ],
                info: info.value(i).to_string(),
                tid: tid.value(i).to_string(),
                dag: dag.value(i),
            };
            result.push(entry);
        }
    }

    Ok(result)
}

fn write_correlation(data: Vec<AdressInfo>) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    // -------------------------------
    // 1. Define schema
    // -------------------------------
    let schema = Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::UInt16, false), // groups row groups
        Field::new("gata", DataType::Utf8, false),         // becomes column chunk inside group
        Field::new("adress", DataType::Utf8, false),       // page-level data
        Field::new("gatunummer", DataType::Utf8, false),   // page-level data
        Field::new("dag", DataType::UInt8, false),
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("lat", DataType::Float64, false),
        Field::new("lon", DataType::Float64, false),
        Field::new("debug_closest_line_id", DataType::Float64, false),
    ]));

    // --------------------------------------------------------------------
    // 2. Group input by postal code → row groups
    // --------------------------------------------------------------------
    let mut grouped: BTreeMap<u16, Vec<AdressInfo>> = BTreeMap::new();

    for d in data {
        grouped.entry(d.postnummer).or_insert_with(Vec::new).push(d);
    }

    // -------------------------------
    // 3. Parquet writer
    // -------------------------------
    let path = "adress_info.parquet".to_string();
    let file = File::create(&path).ok()?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).ok()?;

    // -------------------------------
    // 4. Write each dag group as a row group
    // -------------------------------
    for (postnummer, rows) in grouped {
        let mut post_builder = UInt16Builder::new();
        let mut gata_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut nr_builder = StringBuilder::new();
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut lat_builder = Float64Builder::new();
        let mut lon_builder = Float64Builder::new();
        let mut id_builder = UInt64Builder::new();

        for r in rows {
            post_builder.append_value(postnummer);
            gata_builder.append_value(&r.gata);
            adress_builder.append_value(&r.adress);
            nr_builder.append_value(&r.gatunummer);
            dag_builder.append_value(r.dag);
            tid_builder.append_value(&r.tid);
            info_builder.append_value(&r.info);
            lat_builder.append_value(r.coordinates[0]);
            lon_builder.append_value(r.coordinates[1]);
            id_builder.append_value(r.debug_closest_line_id);
        }

        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(dag_builder.finish())]).ok()?;

        writer.write(&batch).ok()?; // each write() = one row group
    }

    writer.close().ok()?;

    Some(path)
}
