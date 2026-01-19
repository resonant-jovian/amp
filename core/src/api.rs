use geo_types::LineString;
use geojson::{FeatureReader, JsonValue};
use geojson::{GeoJson, Value};
use serde::{Deserialize, Serialize};
use std::fs::read;
use std::io::BufReader;

use crate::structs::{AdressClean, MiljoeDataClean};

#[deprecated]
pub fn read_adresser() -> Vec<AdressClean> {
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

#[deprecated]
pub fn read_miljoeparkering() -> Vec<MiljoeDataClean> {
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
                info,
                tid,
                dag,
            };
            miladr_vec.push(miladr);
        }
    }
    miladr_vec
}

/// ArcGIS Query Response
#[derive(Debug, Deserialize)]
pub struct ArcGISResponse {
    pub features: Vec<ArcGISFeature>,
    #[serde(default)]
    pub exceeded_transfer_limit: bool,
}

/// Individual ArcGIS Feature (raw from API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISFeature {
    pub attributes: JsonValue,
    pub geometry: Option<JsonValue>,
}

/// ArcGIS API Client
pub struct ArcGISClient {
    client: reqwest::Client,
}

impl ArcGISClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch all features from an ArcGIS Feature Service with automatic pagination
    async fn fetch_all_features(
        &self,
        service_url: &str,
        layer_id: u32,
    ) -> Result<Vec<ArcGISFeature>, Box<dyn std::error::Error>> {
        let mut all_features = Vec::new();
        let mut result_offset = 0;
        const RESULT_RECORD_COUNT: u32 = 1000;

        loop {
            let url = format!(
                "{}/{}/query?where=1%3D1&outFields=*&returnGeometry=true&f=json&resultOffset={}&resultRecordCount={}",
                service_url, layer_id, result_offset, RESULT_RECORD_COUNT
            );

            let response: ArcGISResponse = self.client.get(&url).send().await?.json().await?;

            let feature_count = response.features.len();
            all_features.extend(response.features);

            if !response.exceeded_transfer_limit || feature_count < RESULT_RECORD_COUNT as usize {
                break;
            }

            result_offset += RESULT_RECORD_COUNT;
        }

        Ok(all_features)
    }

    /// Convert ArcGIS geometry to GeoJSON and extract point coordinates
    fn extract_point_from_geojson(geometry: &JsonValue) -> Option<[f64; 2]> {
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => match geom.value {
                Value::Point(coords) => {
                    if coords.len() >= 2 {
                        Some([coords[0], coords[1]])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Convert ArcGIS geometry to GeoJSON and extract polygon bounding box
    fn extract_polygon_from_geojson(geometry: &JsonValue) -> Option<[[f64; 2]; 2]> {
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => {
                match geom.value {
                    Value::Polygon(rings) => {
                        if rings.is_empty() || rings.is_empty() {
                            return None;
                        }

                        let ring = &rings[0]; // ✅ Get FIRST ring from rings
                        let first = &ring[0]; // ✅ Get first COORDINATE from that ring
                        let last = &ring[ring.len() - 1]; // ✅ Get last COORDINATE

                        Some([
                            [first[0], first[1]], // ✅ Now index into coordinate [x, y]
                            [last[0], last[1]],
                        ])
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Convert raw ArcGIS features to AdressClean structs
    fn to_adress_clean(&self, features: Vec<ArcGISFeature>) -> Vec<AdressClean> {
        features
            .into_iter()
            .filter_map(|feat| {
                let attrs = &feat.attributes;
                let geometry = feat.geometry?;

                // Extract coordinates from GeoJSON point
                let coordinates = Self::extract_point_from_geojson(&geometry)?;

                // Extract required fields - skip if any are missing
                let postnummer = attrs
                    .get("PostalCode")
                    .or_else(|| attrs.get("postalcode"))
                    .or_else(|| attrs.get("POSTALCODE"))?
                    .as_str()?
                    .parse::<u16>()
                    .ok()?;

                let adress = attrs
                    .get("FullAddress")
                    .or_else(|| attrs.get("Address"))
                    .or_else(|| attrs.get("FULLADDRESS"))?
                    .as_str()?
                    .to_string();

                let gata = attrs
                    .get("StreetName")
                    .or_else(|| attrs.get("Street"))
                    .or_else(|| attrs.get("STREETNAME"))?
                    .as_str()?
                    .to_string();

                let gatunummer = attrs
                    .get("StreetNumber")
                    .or_else(|| attrs.get("Number"))
                    .or_else(|| attrs.get("STREETNUMBER"))?
                    .as_str()?
                    .to_string();

                Some(AdressClean {
                    coordinates,
                    postnummer,
                    adress,
                    gata,
                    gatunummer,
                })
            })
            .collect()
    }

    /// Convert raw ArcGIS features to MiljoeDataClean structs
    fn to_miljoe_clean(&self, features: Vec<ArcGISFeature>) -> Vec<MiljoeDataClean> {
        features
            .into_iter()
            .filter_map(|feat| {
                let attrs = &feat.attributes;
                let geometry = feat.geometry?;

                // Extract bounding box from GeoJSON polygon
                let coordinates = Self::extract_polygon_from_geojson(&geometry)?;

                // Extract required fields - skip if any are missing
                let info = attrs
                    .get("Name")
                    .or_else(|| attrs.get("Info"))
                    .or_else(|| attrs.get("NAME"))?
                    .as_str()?
                    .to_string();

                let tid = attrs
                    .get("Time")
                    .or_else(|| attrs.get("Tid"))
                    .or_else(|| attrs.get("TIME"))?
                    .as_str()?
                    .to_string();

                let dag = attrs
                    .get("Day")
                    .or_else(|| attrs.get("Dag"))
                    .or_else(|| attrs.get("DAY"))?
                    .as_str()?
                    .parse::<u8>()
                    .ok()?;

                Some(MiljoeDataClean {
                    coordinates,
                    info,
                    tid,
                    dag,
                })
            })
            .collect()
    }
}

#[tokio::main]
pub async fn api() -> Result<(Vec<AdressClean>, Vec<MiljoeDataClean>), Box<dyn std::error::Error>> {
    let client = ArcGISClient::new();

    // Malmö Addresses
    println!("Fetching Malmö addresses...");
    let address_features = client
        .fetch_all_features(
            "https://services3.arcgis.com/GVgbJbqm8hXASVYi/ArcGIS/rest/services/Malmo_Sweden_Addresses/FeatureServer",
            0,
        )
        .await?;

    let addresses = client.to_adress_clean(address_features);
    println!("Converted {} raw features to AdressClean", addresses.len());

    // Print first few entries
    for (i, addr) in addresses.iter().take(3).enumerate() {
        println!("  [{}] {} ({})", i + 1, addr.adress, addr.postnummer);
    }

    // Miljö Parking
    println!("\nFetching environmental parking data...");
    let parking_features = client
        .fetch_all_features(
            "https://gis.malmo.se/arcgis/rest/services/FGK_Parkster_Map/FeatureServer",
            1,
        )
        .await?;

    let parking = client.to_miljoe_clean(parking_features);
    println!(
        "Converted {} raw features to MiljoeDataClean",
        parking.len()
    );

    // Print first few entries
    for (i, park) in parking.iter().take(3).enumerate() {
        println!("  [{}] {} ({})", i + 1, park.info, park.dag);
    }

    // Now you have:
    // - addresses: Vec<AdressClean>
    // - parking: Vec<MiljoeDataClean>
    // Use them directly - no file I/O

    Ok((addresses, parking))
}
