use geojson::JsonValue;
use geojson::{GeoJson, Value};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::structs::{AdressClean, MiljoeDataClean};

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
    fn extract_point_from_geojson(geometry: &JsonValue) -> Option<[Decimal; 2]> {
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => match geom.value {
                Value::Point(coords) => {
                    if coords.len() >= 2 {
                        // Parse f64 from GeoJSON, convert to Decimal with full precision
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

    /// Convert ArcGIS geometry to GeoJSON and extract polygon bounding box
    fn extract_polygon_from_geojson(geometry: &JsonValue) -> Option<[[Decimal; 2]; 2]> {
        let geom_json = serde_json::to_string(geometry).ok()?;
        match geom_json.parse::<GeoJson>() {
            Ok(GeoJson::Geometry(geom)) => {
                match geom.value {
                    Value::Polygon(rings) => {
                        if rings.is_empty() || rings[0].is_empty() {
                            return None;
                        }

                        let ring = &rings[0];
                        let first = &ring[0];
                        let last = &ring[ring.len() - 1];

                        let first_x = Decimal::from_f64_retain(first[0]).unwrap_or_default();
                        let first_y = Decimal::from_f64_retain(first[1]).unwrap_or_default();
                        let last_x = Decimal::from_f64_retain(last[0]).unwrap_or_default();
                        let last_y = Decimal::from_f64_retain(last[1]).unwrap_or_default();

                        Some([
                            [first_x, first_y],
                            [last_x, last_y],
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
