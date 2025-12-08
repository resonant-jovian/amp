use geojson::FeatureReader;
use nestify::nest;
use serde::{Deserialize, Serialize};
use std::fs::{File, read};
use std::io::{BufReader, PipeReader, Read};

nest! {
    #[derive(Debug, Deserialize)]
    struct AdressDirty {
        r#type: String,
        geometry: #[derive(Debug, Deserialize)] struct Geometry {
            r#type: String,
            coordinates: [f64; 2],
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

#[derive(Serialize)]
struct AdressClean {
    coordinates: [f64; 2],
    postnummer: String,
    adress: String,
    gata: String,
    gatunummer: String, //usize?
}

fn main() {
    let feature_collection_string = r#"{
     "type": "FeatureCollection",
     "features": [
         {
           "type": "Feature",
           "geometry": { "type": "Point", "coordinates": [125.6, 10.1] },
           "properties": {
             "name": "Dinagat Islands",
             "age": 123
           }
         },
         {
           "type": "Feature",
           "geometry": { "type": "Point", "coordinates": [2.3, 4.5] },
           "properties": {
             "name": "Neverland",
             "age": 456
           }
         }
     ]
}"#
    .as_bytes();
    let io_reader = std::io::BufReader::new(feature_collection_string);
    let file = read("adresser.json").unwrap();
    let reader = BufReader::new(file.as_slice());
    let feature_reader = FeatureReader::from_reader(reader);
    for feature in feature_reader.features() {
        let feature = feature.expect("valid geojson feature");

        let c = feature.property("coordinates").unwrap().as_array().unwrap();

        let postnummer = feature
            .property("postnr")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let adress = feature
            .property("beladress")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let gata = feature
            .property("adressomr")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let gatunummer = feature
            .property("adressplat")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let coordinates = c.as_slice();
        println!("{:?}", coordinates);
        println!("{}", postnummer);
        println!("{}", adress);
        println!("{}", gata);
        println!("{}", gatunummer);
    }
}
