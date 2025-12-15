use arrow::array::*;
use arrow::datatypes::*;
use arrow::datatypes::{Field, Schema};
use arrow::record_batch::RecordBatch;
use geo_types::LineString;
use geojson::{FeatureReader, JsonValue};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::{EnabledStatistics, WriterProperties};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::read;
use std::io::BufReader;
use std::sync::Arc;

//use nestify::nest;
//Below struct is structure example for adresser.geojson
/*
nest! {
    #[derive(Debug, Deserialize)]
    struct AdressDirty {
        r#type: String,
        geometry: #[derive(Debug, Deserialize)] struct Geometry {
            r#type: String,
            coordinates: geo_types::Point, //([f64; 2] with declared point feature)
        },
        properties: #[derive(Debug, Deserialize)] struct Properties {
            ogc_fid: usize,
            beladress: String,
            popnamn: String, //Null
            postnr: String,
            postort: String,
            adressomr: String,
            adressplat: String,
            nr_num: usize,
            nr_litt: String, //Null
            object_id: usize, //ObjectId
        },
    }
}
*/
#[derive(Serialize, Debug)]
pub struct AdressClean {
    coordinates: [f64; 2], //lat (x), long (y)
    postnummer: u16,
    adress: String,
    gata: String,
    gatunummer: String, //usize? No, 9B ex.
}

#[derive(Serialize, Debug)]
#[derive(Clone)]
pub struct MiljoeDataClean {
    coordinates: [[f64; 2]; 2], //start -> end (lat (x), long (y))
    //extra_info: String,
    info: String,
    tid: String,
    dag: u8,
}

fn main() {
    write_adresser(read_adresser());
    write_miljoeparkering(read_miljoeparkering());
}
fn read_adresser(/*Path?*/) -> Vec<AdressClean> {
    let mut adr_vec = vec![];
    let file = read("adresser.geojson").expect("failed to read file");
    let reader = BufReader::new(file.as_slice());
    let feature_reader = FeatureReader::from_reader(reader);
    for feature in feature_reader.features() {
        let feature = feature.expect("failed to iterate over valid geojson feature");
        if feature.geometry.is_some()
            && feature.contains_property("postnr")
            && feature.contains_property("beladress")
            && feature.contains_property("adressomr")
            && feature.contains_property("adressplat")
        {
            let mut postnummer: u16 = Default::default();
            match feature.property("postnr") {
                Some(feature) if feature.clone() == JsonValue::Null => {}
                Some(feature) => {
                    postnummer = feature
                        .as_str()
                        .expect("failed to turn postnummer to &str")
                        .replace(" ", "")
                        .parse()
                        .expect("failed to turn &str to u16");
                }
                None => {}
            }
            let adress = feature
                .property("beladress")
                .expect("failed to get adress property")
                .as_str()
                .expect("failed to turn adress to &str")
                .to_string();
            let gata = feature
                .property("adressomr")
                .expect("failed to get gata property")
                .as_str()
                .expect("failed to turn gata to &str")
                .to_string();
            let gatunummer = feature
                .property("adressplat")
                .expect("failed to get gatunummer property")
                .as_str()
                .expect("failed to turn gatunummer to &str")
                .to_string();
            let c = feature.geometry.expect("failed to extract geometry").value; //Extract coords
            let c_type: geo_types::Point = c.try_into().expect("failed to convert coordinates");
            let coordinates = [c_type.x(), c_type.y()];
            let adr = AdressClean {
                coordinates,
                postnummer,
                adress,
                gata,
                gatunummer,
            };
            adr_vec.push(adr);
        }
    }
    adr_vec
}

fn read_miljoeparkering() -> Vec<MiljoeDataClean> {
    let mut miladr_vec = vec![];
    let file = read("miljöparkering.geojson").expect("failed to read file");
    let reader = BufReader::new(file.as_slice());
    let feature_reader = FeatureReader::from_reader(reader);
    for feature in feature_reader.features() {
        let feature = feature.expect("failed to iterate over valid geojson feature");
        if feature.geometry.is_some()
            && feature.contains_property("value")
            && feature.contains_property("copy_value")
            && feature.contains_property("tiden")
            && feature.contains_property("day")
        {
            /*
            let extra_info = feature
                .property("value")
                .expect("failed to get extra_info property")
                .as_str()
                .expect("failed to turn extra_info to &str")
                .to_string();

             */
            let info = feature
                .property("copy_value")
                .expect("failed to get info property")
                .as_str()
                .expect("failed to turn info to &str")
                .to_string();
            let tid = feature
                .property("tiden")
                .expect("failed to get tid property")
                .as_str()
                .expect("failed to turn tid to &str")
                .to_string();
            let dag: u8 = feature
                .property("day")
                .expect("failed to get dag property")
                .as_str()
                .expect("failed to turn dag to &str")
                .parse()
                .expect("failed to turn &str to u8");
            let c = feature.geometry.expect("failed to extract geometry").value; //Extract coords
            let c_type: LineString = c.try_into().expect("failed to convert coordinates");
            let c_init = c_type.into_points();
            let start = c_init.iter().next().expect("failed to extract start");
            let end = c_init.iter().next().expect("failed to extract end");
            let coordinates = [[start.x(), start.y()], [end.x(), end.y()]];
            let miladr = MiljoeDataClean {
                coordinates,
                //extra_info,
                info,
                tid,
                dag,
            };
            miladr_vec.push(miladr);
        }
    }
    miladr_vec
}

pub fn write_adresser(data: Vec<AdressClean>) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    // --------------------------------------------------------------------
    // 1. Define schema (gata becomes column chunk, others page data)
    // --------------------------------------------------------------------
    let schema = Arc::new(Schema::new(vec![
        Field::new("postnummer", DataType::UInt16, false), // groups row groups
        Field::new("gata", DataType::Utf8, false),         // becomes column chunk inside group
        Field::new("adress", DataType::Utf8, false),       // page-level data
        Field::new("gatunummer", DataType::Utf8, false),   // page-level data
        Field::new("lat", DataType::Float64, false),
        Field::new("lon", DataType::Float64, false),
    ]));

    // --------------------------------------------------------------------
    // 2. Group input by postal code → row groups
    // --------------------------------------------------------------------
    let mut grouped: BTreeMap<u16, Vec<AdressClean>> = BTreeMap::new();

    for d in data {
        grouped.entry(d.postnummer).or_insert_with(Vec::new).push(d);
    }

    // --------------------------------------------------------------------
    // 3. Set up Parquet writer
    // --------------------------------------------------------------------
    let path = "adresser.parquet".to_string();
    let file = std::fs::File::create(&path).expect("Failed to create file");

    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .set_max_row_group_size(1024 * 1024) // row group ≈1MB target (ignored since we force our own groups)
        .build();

    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).expect("Failed to create Arrow writer");

    // --------------------------------------------------------------------
    // 4. Convert each postal code group into a row group
    // --------------------------------------------------------------------
    for (postnummer, rows) in grouped {
        let mut post_builder = UInt16Builder::new();
        let mut gata_builder = StringBuilder::new();
        let mut adress_builder = StringBuilder::new();
        let mut nr_builder = StringBuilder::new();
        let mut lat_builder = Float64Builder::new();
        let mut lon_builder = Float64Builder::new();

        for r in rows {
            post_builder.append_value(postnummer);
            gata_builder.append_value(&r.gata);
            adress_builder.append_value(&r.adress);
            nr_builder.append_value(&r.gatunummer);
            lat_builder.append_value(r.coordinates[0]);
            lon_builder.append_value(r.coordinates[1]);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(post_builder.finish()),
                Arc::new(gata_builder.finish()),
                Arc::new(adress_builder.finish()),
                Arc::new(nr_builder.finish()),
                Arc::new(lat_builder.finish()),
                Arc::new(lon_builder.finish()),
            ],
        )
        .expect("Failed to create record batch");

        writer.write(&batch).expect("Failed to write batch"); // ← **each write() = one row group**
    }

    writer.close().expect("Failed to close writer");
    Some(path)
}
pub fn write_miljoeparkering(data: Vec<MiljoeDataClean>) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    // -------------------------------
    // 1. Define schema
    // -------------------------------
    let schema = Arc::new(Schema::new(vec![
        Field::new("dag", DataType::UInt8, false), // row group
        Field::new("tid", DataType::Utf8, false),
        Field::new("info", DataType::Utf8, false),
        Field::new("lat_start", DataType::Float64, false),
        Field::new("lon_start", DataType::Float64, false),
        Field::new("lat_end", DataType::Float64, false),
        Field::new("lon_end", DataType::Float64, false),
        Field::new("id", DataType::UInt16, false),
    ]));

    // -------------------------------
    // 2. Group by dag → row groups
    // -------------------------------
    let mut grouped: BTreeMap<u8, Vec<MiljoeDataClean>> = BTreeMap::new();
    for d in data {
        grouped.entry(d.dag).or_insert_with(Vec::new).push(d);
    }

    // -------------------------------
    // 3. Parquet writer
    // -------------------------------
    let path = "miljöparkering.parquet".to_string();
    let file = std::fs::File::create(&path).expect("Failed to create file");
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props)).expect("Failed to create Arrow writer");

    // -------------------------------
    // 4. Write each dag group as a row group
    // -------------------------------
    let mut i: u16 = 1;
    for (dag, rows) in grouped {
        let mut dag_builder = UInt8Builder::new();
        let mut tid_builder = StringBuilder::new();
        let mut info_builder = StringBuilder::new();
        let mut lat_start_builder = Float64Builder::new();
        let mut lon_start_builder = Float64Builder::new();
        let mut lat_end_builder = Float64Builder::new();
        let mut lon_end_builder = Float64Builder::new();
        let mut id_builder = UInt16Builder::new();

        for r in rows {
            dag_builder.append_value(dag);
            tid_builder.append_value(&r.tid);
            info_builder.append_value(&r.info);
            lat_start_builder.append_value(r.coordinates[0][0]);
            lon_start_builder.append_value(r.coordinates[0][1]);
            lat_end_builder.append_value(r.coordinates[1][0]);
            lon_end_builder.append_value(r.coordinates[1][1]);
            id_builder.append_value(i);
            i += 1;
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(dag_builder.finish()),
                Arc::new(tid_builder.finish()),
                Arc::new(info_builder.finish()),
                Arc::new(lat_start_builder.finish()),
                Arc::new(lon_start_builder.finish()),
                Arc::new(lat_end_builder.finish()),
                Arc::new(lon_end_builder.finish()),
                Arc::new(id_builder.finish()),
            ],
        )
        .expect("Failed to create RecordBatch");

        writer.write(&batch).expect("Failed to write batch"); // each write() = one row group
    }

    writer.close().expect("Failed to close writer");
    Some(path)
}
