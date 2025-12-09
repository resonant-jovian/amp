use geo_types;
use geojson::{FeatureReader, JsonValue};
use serde::Serialize;
use std::fs::read;
use std::io::BufReader;
use geo_types::LineString;

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
struct AdressClean {
    coordinates: [f64; 2], //lat (x), long (y)
    postnummer: String,
    adress: String,
    gata: String,
    gatunummer: String, //usize? No, 9B ex.
}

#[derive(Serialize, Debug)]
struct MiljöDataClean {
    coordinates: [[f64; 2]; 2], //start -> end (lat (x), long (y))
    //extra_info: String,
    info: String,
    tid: String,
    dag: String, //usize?
}

fn main() {
    println!("{:?}", read_adresser());
    println!("{:?}", read_miljöparkering())
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
            let mut postnummer = "".to_string();
            match feature.property("postnr") {
                Some(feature) if feature.clone() == JsonValue::Null => {}
                Some(feature) => {
                    postnummer = feature
                        .as_str() //Some str conv not working
                        .expect("failed to turn postnummer to &str")
                        .to_string();
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
            //println!("{:?}", adr);
            adr_vec.push(adr);
        }
    }
    adr_vec
}

fn read_miljöparkering() -> Vec<MiljöDataClean>{
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
            let dag = feature
                .property("day")
                .expect("failed to get dag property")
                .as_str()
                .expect("failed to turn dag to &str")
                .to_string();
            let c = feature.geometry.expect("failed to extract geometry").value; //Extract coords
            let c_type: LineString = c.try_into().expect("failed to convert coordinates");
            let c_init = c_type.into_points();
            let start = c_init.iter().next().expect("failed to extract start");
            let end = c_init.iter().next().expect("failed to extract end");
            let coordinates = [[start.x(), start.y()], [end.x(), end.y()]];
            let miladr = MiljöDataClean {
                coordinates,
                //extra_info,
                info,
                tid,
                dag,
            };
            //println!("{:?}", miladr);
            miladr_vec.push(miladr);
        }
    }
    miladr_vec
}
