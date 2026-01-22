use crate::structs::{AdressClean, MiljoeDataClean};
use geodesy::prelude::*;
use geojson::JsonValue;
use geojson::{GeoJson, Value};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcGISFeature {
    pub attributes: JsonValue,
    pub geometry: Option<JsonValue>,
}

pub struct ArcGISClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct ArcGISResponse {
    pub features: Vec<ArcGISFeature>,

    #[serde(default, rename = "exceededTransferLimit")] // ← Add this rename
    pub exceeded_transfer_limit: bool,
}

impl ArcGISClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn convert_web_mercator_to_sweref99tm(
        x: f64,
        y: f64,
    ) -> Result<(Decimal, Decimal), Box<dyn std::error::Error>> {
        let mut context = Minimal::new();

        let op = context.op("cart  +xy:in +xy:out | \
         adapt +xy:proj=merc +R=6378137 +xy:in | \
         adapt +xy:proj=tmerc lon_0=15 k=0.9996 x_0=500000 y_0=0 +xy:out")?;

        let mut data = [Coor2D::raw(x, y)];
        context.apply(op, Fwd, &mut data)?;

        // Convert to Decimal, round to 7 decimal places (±1.1cm accuracy)
        let x_result = Decimal::from_f64_retain(data[0][0])
            .ok_or("Failed to convert x to Decimal")?
            .round_dp(7);

        let y_result = Decimal::from_f64_retain(data[0][1])
            .ok_or("Failed to convert y to Decimal")?
            .round_dp(7);

        Ok((x_result, y_result))
    }

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

            println!("Fetching: {}", url); // ← ADD THIS

            let response: ArcGISResponse = self.client.get(&url).send().await?.json().await?;

            println!("Received {} features", response.features.len()); // ← ADD THIS

            let feature_count = response.features.len();
            all_features.extend(response.features);

            if !response.exceeded_transfer_limit || feature_count < RESULT_RECORD_COUNT as usize {
                break;
            }

            result_offset += RESULT_RECORD_COUNT;
        }

        println!("Total features collected: {}", all_features.len()); // ← ADD THIS
        Ok(all_features)
    }

    fn extract_point_from_geojson(geometry: &JsonValue) -> Option<[Decimal; 2]> {
        // Try ArcGIS Multipoint format first (points array)
        if let Some(points_array) = geometry.get("points") {
            if let Some(points) = points_array.as_array() {
                if let Some(first_point) = points.first() {
                    if let Some(coords) = first_point.as_array() {
                        if coords.len() >= 2 {
                            let x =
                                Decimal::from_f64_retain(coords[0].as_f64()?).unwrap_or_default();
                            let y =
                                Decimal::from_f64_retain(coords[1].as_f64()?).unwrap_or_default();
                            return Some([x, y]);
                        }
                    }
                }
            }
        }

        // Try ArcGIS Point format (x, y) - for other endpoints
        if let (Some(x), Some(y)) = (geometry.get("x"), geometry.get("y")) {
            let x_val = Decimal::from_f64_retain(x.as_f64()?).unwrap_or_default();
            let y_val = Decimal::from_f64_retain(y.as_f64()?).unwrap_or_default();
            return Some([x_val, y_val]);
        }

        // Fallback to GeoJSON format
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => match geom.value {
                Value::Point(coords) => {
                    if coords.len() >= 2 {
                        let x = Decimal::from_f64_retain(coords[0]).unwrap_or_default();
                        let y = Decimal::from_f64_retain(coords[1]).unwrap_or_default();
                        Some([x, y])
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn extract_polyline_from_geojson(geometry: &JsonValue) -> Option<[[Decimal; 2]; 2]> {
        // Try ArcGIS polyline format (paths array)
        if let Some(paths) = geometry.get("paths") {
            if let Some(paths_array) = paths.as_array() {
                if !paths_array.is_empty() {
                    if let Some(first_path) = paths_array[0].as_array() {
                        if first_path.len() >= 2 {
                            // Get first point
                            let first_pt = &first_path[0];
                            let first_x =
                                Decimal::from_f64_retain(first_pt[0].as_f64()?).unwrap_or_default();
                            let first_y =
                                Decimal::from_f64_retain(first_pt[1].as_f64()?).unwrap_or_default();

                            // Get last point
                            let last_pt = &first_path[first_path.len() - 1];
                            let last_x =
                                Decimal::from_f64_retain(last_pt[0].as_f64()?).unwrap_or_default();
                            let last_y =
                                Decimal::from_f64_retain(last_pt[1].as_f64()?).unwrap_or_default();

                            return Some([[first_x, first_y], [last_x, last_y]]);
                        }
                    }
                }
            }
        }

        // Fallback to GeoJSON format
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => match geom.value {
                Value::LineString(coords) => {
                    if coords.len() < 2 {
                        return None;
                    }
                    let first = &coords[0];
                    let last = &coords[coords.len() - 1];
                    let first_x = Decimal::from_f64_retain(first[0]).unwrap_or_default();
                    let first_y = Decimal::from_f64_retain(first[1]).unwrap_or_default();
                    let last_x = Decimal::from_f64_retain(last[0]).unwrap_or_default();
                    let last_y = Decimal::from_f64_retain(last[1]).unwrap_or_default();
                    Some([[first_x, first_y], [last_x, last_y]])
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn to_adress_clean(&self, features: Vec<ArcGISFeature>) -> Vec<AdressClean> {
        let mut converted = 0;
        let mut no_geometry = 0;
        let mut extraction_failed = 0;

        let result: Vec<_> = features
            .into_iter()
            .filter_map(|feat| {
                let attrs = &feat.attributes;
                let geometry = match feat.geometry {
                    Some(g) => g,
                    None => {
                        no_geometry += 1;
                        return None;
                    }
                };

                let coordinates = match Self::extract_point_from_geojson(&geometry) {
                    Some(c) => {
                        let x_f64 = c[0].to_f64().unwrap_or(0.0);
                        let y_f64 = c[1].to_f64().unwrap_or(0.0);

                        match Self::convert_web_mercator_to_sweref99tm(x_f64, y_f64) {
                            Ok((x_sweref, y_sweref)) => [x_sweref, y_sweref],
                            Err(e) => {
                                eprintln!("Projection error: {}", e);
                                extraction_failed += 1;
                                return None;
                            }
                        }
                    }
                    None => {
                        extraction_failed += 1;
                        return None;
                    }
                };

                let postnummer = attrs.get("postnr")?.as_str()?.to_string();
                let adress = attrs.get("beladress")?.as_str()?.to_string();
                let gata = attrs.get("adressomr")?.as_str()?.to_string();
                let gatunummer = attrs.get("adressplat")?.as_str()?.to_string();

                converted += 1;

                Some(AdressClean {
                    coordinates,
                    postnummer,
                    adress,
                    gata,
                    gatunummer,
                })
            })
            .collect();

        println!(
            " Conversion stats - No geometry: {}, Extraction failed: {}, Converted: {}",
            no_geometry, extraction_failed, converted
        );

        result
    }

    fn to_miljoe_clean(&self, features: Vec<ArcGISFeature>) -> Vec<MiljoeDataClean> {
        let mut converted = 0;
        let mut no_geometry = 0;
        let mut extraction_failed = 0;

        let result: Vec<_> = features
            .into_iter()
            .filter_map(|feat| {
                let attrs = &feat.attributes;

                let geometry = match feat.geometry {
                    Some(g) => g,
                    None => {
                        no_geometry += 1;
                        return None;
                    }
                };

                let coordinates = match Self::extract_polyline_from_geojson(&geometry) {
                    Some(c) => c,
                    None => {
                        extraction_failed += 1;
                        return None;
                    }
                };

                let info = attrs
                    .get("value")
                    .or_else(|| attrs.get("Info"))
                    .or_else(|| attrs.get("NAME"))?
                    .as_str()?
                    .to_string();

                let tid = attrs
                    .get("tiden")
                    .or_else(|| attrs.get("Tid"))
                    .or_else(|| attrs.get("TIME"))?
                    .as_str()?
                    .to_string();

                let dag = attrs
                    .get("day")
                    .or_else(|| attrs.get("Dag"))
                    .or_else(|| attrs.get("DAY"))?
                    .as_str()?
                    .parse::<u8>()
                    .ok()?;

                converted += 1;
                Some(MiljoeDataClean {
                    coordinates,
                    info,
                    tid,
                    dag,
                })
            })
            .collect();

        println!(
            "  Conversion stats - No geometry: {}, Extraction failed: {}, Converted: {}",
            no_geometry, extraction_failed, converted
        );

        result
    }
}

pub async fn api() -> Result<(Vec<AdressClean>, Vec<MiljoeDataClean>), Box<dyn std::error::Error>> {
    let client = ArcGISClient::new();

    println!("Fetching Malmö addresses...");
    let address_features = client
        .fetch_all_features(
            "https://services3.arcgis.com/GVgbJbqm8hXASVYi/ArcGIS/rest/services/Malmo_Sweden_Addresses/FeatureServer",
            0,
        )
        .await?;

    let addresses = client.to_adress_clean(address_features);
    println!("Converted {} raw features to AdressClean", addresses.len());

    for (i, addr) in addresses.iter().take(3).enumerate() {
        println!("  [{}] {} ({})", i + 1, addr.adress, addr.postnummer);
    }

    println!("\nFetching environmental parking data...");
    let parking_features = client
        .fetch_all_features(
            "https://gis.malmo.se/arcgis/rest/services/FGK/Parkster/MapServer",
            1,
        )
        .await?;

    let parking = client.to_miljoe_clean(parking_features);
    println!(
        "Converted {} raw features to MiljoeDataClean",
        parking.len()
    );

    for (i, park) in parking.iter().take(3).enumerate() {
        println!("  [{}] {} ({})", i + 1, park.info, park.dag);
    }

    Ok((addresses, parking))
}
