use crate::structs::{AdressClean, MiljoeDataClean};
use geojson::{Feature, GeoJson};
use rust_decimal::Decimal;
use std::fs;

pub struct DataLoader;

impl DataLoader {
    pub fn new() -> Self {
        Self
    }

    fn extract_point_coordinates(feature: &Feature) -> Option<[Decimal; 2]> {
        if let Some(ref geom) = feature.geometry {
            match &geom.value {
                geojson::Value::Point(coords) => {
                    if coords.len() >= 2 {
                        let x = Decimal::try_from(coords[0]).ok()?;
                        let y = Decimal::try_from(coords[1]).ok()?;
                        return Some([x, y]);
                    }
                }
                _ => return None,
            }
        }
        None
    }

    fn extract_linestring_endpoints(feature: &Feature) -> Option<[[Decimal; 2]; 2]> {
        if let Some(ref geom) = feature.geometry {
            match &geom.value {
                geojson::Value::LineString(coords) => {
                    if coords.len() >= 2 {
                        let first = &coords[0];
                        let last = &coords[coords.len() - 1];

                        let x1 = Decimal::try_from(first[0]).ok()?;
                        let y1 = Decimal::try_from(first[1]).ok()?;
                        let x2 = Decimal::try_from(last[0]).ok()?;
                        let y2 = Decimal::try_from(last[1]).ok()?;

                        return Some([[x1, y1], [x2, y2]]);
                    }
                }
                geojson::Value::MultiLineString(lines) => {
                    if !lines.is_empty() && lines[0].len() >= 2 {
                        let first_line = &lines[0];
                        let first = &first_line[0];
                        let last = &first_line[first_line.len() - 1];

                        let x1 = Decimal::try_from(first[0]).ok()?;
                        let y1 = Decimal::try_from(first[1]).ok()?;
                        let x2 = Decimal::try_from(last[0]).ok()?;
                        let y2 = Decimal::try_from(last[1]).ok()?;

                        return Some([[x1, y1], [x2, y2]]);
                    }
                }
                _ => return None,
            }
        }
        None
    }

    fn parse_address_feature(feature: Feature) -> Option<AdressClean> {
        let props = feature.clone().properties?;

        let coordinates = Self::extract_point_coordinates(&feature)?;

        let postnummer = props.get("POSTNR")?.as_str()?.to_string();
        let adress = props.get("BELADRESS")?.as_str()?.to_string();
        let gata = props.get("ADRESSOMR")?.as_str()?.to_string();
        let gatunummer = props.get("ADRESSPLAT")?.as_str()?.to_string();

        Some(AdressClean {
            coordinates,
            postnummer,
            adress,
            gata,
            gatunummer,
        })
    }

    fn parse_parking_feature(feature: Feature) -> Option<MiljoeDataClean> {
        let props = feature.clone().properties?;

        let coordinates = Self::extract_linestring_endpoints(&feature)?;

        // Get info from 'value' or 'copyvalue' field
        let info = props
            .get("value")
            .or_else(|| props.get("copyvalue"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Get time info
        let tid = props
            .get("tid")
            .and_then(|v| v.as_str())
            .unwrap_or("00:00 - 00:00")
            .to_string();

        // Get day info
        let dag = props
            .get("day")
            .and_then(|v| {
                if let Some(num) = v.as_u64() {
                    Some(num as u8)
                } else if let Some(s) = v.as_str() {
                    s.parse::<u8>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        Some(MiljoeDataClean {
            coordinates,
            info,
            tid,
            dag,
        })
    }

    pub fn load_addresses(path: &str) -> Result<Vec<AdressClean>, Box<dyn std::error::Error>> {
        println!("Loading addresses from: {}", path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let addresses: Vec<AdressClean> = if let GeoJson::FeatureCollection(collection) = geojson {
            collection
                .features
                .into_iter()
                .filter_map(Self::parse_address_feature)
                .collect()
        } else {
            return Err("Invalid GeoJSON format for addresses".into());
        };

        println!("Loaded {} addresses", addresses.len());

        // Show sample
        for (i, addr) in addresses.iter().take(3).enumerate() {
            println!("  [{}] {} ({})", i + 1, addr.adress, addr.postnummer);
        }

        Ok(addresses)
    }

    pub fn load_parking(
        path: &str,
        dataset_name: &str,
    ) -> Result<Vec<MiljoeDataClean>, Box<dyn std::error::Error>> {
        println!("\nLoading {} from: {}", dataset_name, path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let parking: Vec<MiljoeDataClean> = if let GeoJson::FeatureCollection(collection) = geojson
        {
            collection
                .features
                .into_iter()
                .filter_map(Self::parse_parking_feature)
                .collect()
        } else {
            return Err(format!("Invalid GeoJSON format for {}", dataset_name).into());
        };

        println!("Loaded {} {}", parking.len(), dataset_name);

        // Show sample
        for (i, park) in parking.iter().take(3).enumerate() {
            println!("  [{}] {} (Day {})", i + 1, park.info, park.dag);
        }

        Ok(parking)
    }
}

pub fn api() -> Result<
    (Vec<AdressClean>, Vec<MiljoeDataClean>, Vec<MiljoeDataClean>),
    Box<dyn std::error::Error>,
> {
    let _loader = DataLoader::new();

    // Load from local JSON files (Malmö open data)
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_parking("data/miljoparkeringar.json", "Miljödata")?;
    let parkering = DataLoader::load_parking("data/parkeringsavgifter.json", "Parkering avgifter")?;

    println!("\n✓ Data loading complete");
    println!("  Total addresses: {}", addresses.len());
    println!("  Total miljödata zones: {}", miljodata.len());
    println!("  Total parkering zones: {}", parkering.len());

    Ok((addresses, miljodata, parkering))
}

pub fn api_miljo_only()
-> Result<(Vec<AdressClean>, Vec<MiljoeDataClean>), Box<dyn std::error::Error>> {
    let _loader = DataLoader::new();
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_parking("data/miljoparkeringar.json", "Miljödata")?;
    Ok((addresses, miljodata))
}
