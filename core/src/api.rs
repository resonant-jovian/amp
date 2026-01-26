use crate::structs::{AdressClean, MiljoeDataClean};
use geojson::{Feature, GeoJson};
use rust_decimal::Decimal;
use std::fs;

pub type ApiResult = (Vec<AdressClean>, Vec<MiljoeDataClean>, Vec<MiljoeDataClean>);

pub struct DataLoader;

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

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

    fn extract_time_from_taxa(taxa_str: &str) -> String {
        // Look for pattern like "8–22" or "8–20" (with en-dash U+2013)
        // Split on en-dash and extract the time range
        let parts: Vec<&str> = taxa_str.split('–').collect();

        if parts.len() >= 2 {
            // Find digits before and after the dash
            if let Some(before_dash) = parts.first() {
                // Get the last number from before the dash (the start time)
                let start_time = before_dash.split_whitespace().last().and_then(|s| {
                    s.chars()
                        .rev()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>()
                        .parse::<u32>()
                        .ok()
                });

                if let Some(start) = start_time
                    && let Some(after_dash) = parts.get(1)
                {
                    // Get the first number after the dash (the end time)
                    let end_time = after_dash
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u32>()
                        .ok();

                    if let Some(end) = end_time {
                        return format!("{:02}:00–{:02}:00", start, end);
                    }
                }
            }
        }

        "00:00–23:59".to_string()
    }

    fn parse_parking_feature(feature: Feature, is_avgifter: bool) -> Option<MiljoeDataClean> {
        let props = feature.clone().properties?;

        let coordinates = Self::extract_linestring_endpoints(&feature)?;

        // For parkeringsavgifter (fee data), use 'taxa' field; for miljöparkeringar, try 'value'/'copyvalue'
        let info = if is_avgifter {
            // For avgifter, use taxa field which contains parking rate info
            props
                .get("taxa")
                .or_else(|| props.get("value"))
                .or_else(|| props.get("copyvalue"))
                .and_then(|v| v.as_str())
                .unwrap_or("Okänd")
                .to_string()
        } else {
            // For miljöparkeringar, try value/copyvalue
            props
                .get("value")
                .or_else(|| props.get("copyvalue"))
                .and_then(|v| v.as_str())
                .unwrap_or("Okänd")
                .to_string()
        };

        // Get time info - avgifter typically have it, miljöparkeringar may not
        let tid = props
            .get("tid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // For avgifter, try to extract time from taxa field
                if is_avgifter {
                    if let Some(taxa_str) = props.get("taxa").and_then(|v| v.as_str()) {
                        Self::extract_time_from_taxa(taxa_str)
                    } else {
                        "00:00–23:59".to_string()
                    }
                } else {
                    "00:00–23:59".to_string()
                }
            });

        // Get day info - for avgifter it's always all days (0 means all days in context)
        // For miljöparkeringar it may be specific days
        let dag = if is_avgifter {
            0u8 // 0 = all days for parking fees
        } else {
            props
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
                .unwrap_or(0)
        };

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

        // Determine if this is avgifter (fees) data
        let is_avgifter = dataset_name.to_lowercase().contains("avgift");

        let parking: Vec<MiljoeDataClean> = if let GeoJson::FeatureCollection(collection) = geojson
        {
            collection
                .features
                .into_iter()
                .filter_map(|f| Self::parse_parking_feature(f, is_avgifter))
                .collect()
        } else {
            return Err(format!("Invalid GeoJSON format for {}", dataset_name).into());
        };

        println!("Loaded {} {}", parking.len(), dataset_name);

        // Show sample
        for (i, park) in parking.iter().take(3).enumerate() {
            if is_avgifter {
                println!("  [{}] {} ({})", i + 1, park.info, park.tid);
            } else {
                println!("  [{}] {} (Day {})", i + 1, park.info, park.dag);
            }
        }

        Ok(parking)
    }
}

pub fn api() -> Result<ApiResult, Box<dyn std::error::Error>> {
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
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_parking("data/miljoparkeringar.json", "Miljödata")?;
    Ok((addresses, miljodata))
}
