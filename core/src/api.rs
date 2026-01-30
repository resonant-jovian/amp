use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use geojson::{Feature, GeoJson};
use rust_decimal::Decimal;
use std::fs;
pub type ApiResult = (Vec<AdressClean>, Vec<MiljoeDataClean>, Vec<ParkeringsDataClean>);
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
    /// Extract all line segments from a geometry (handles both LineString and MultiLineString)
    /// Returns a Vec of [start, end] coordinate pairs, one per segment
    fn extract_all_line_segments(feature: &Feature) -> Option<Vec<[[Decimal; 2]; 2]>> {
        let mut segments = Vec::new();
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
                        segments.push([[x1, y1], [x2, y2]]);
                    }
                }
                geojson::Value::MultiLineString(lines) => {
                    for line in lines {
                        if line.len() >= 2 {
                            let first = &line[0];
                            let last = &line[line.len() - 1];
                            let x1 = Decimal::try_from(first[0]).ok()?;
                            let y1 = Decimal::try_from(first[1]).ok()?;
                            let x2 = Decimal::try_from(last[0]).ok()?;
                            let y2 = Decimal::try_from(last[1]).ok()?;
                            segments.push([[x1, y1], [x2, y2]]);
                        }
                    }
                }
                _ => return None,
            }
        }
        if segments.is_empty() {
            None
        } else {
            Some(segments)
        }
    }
    fn _extract_point_coordinates_legacy(feature: &Feature) -> Option<[Decimal; 2]> {
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
    fn parse_address_feature(feature: Feature) -> Option<AdressClean> {
        let props = feature.clone().properties?;
        let coordinates = Self::extract_point_coordinates(&feature)?;

        // Handle optional postnummer
        let postnummer = props
            .get("POSTNR")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

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
        let parts: Vec<&str> = taxa_str.split('–').collect();
        if parts.len() >= 2
            && let Some(before_dash) = parts.first()
        {
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
        "00:00–23:59".to_string()
    }
    /// Parse a parking feature and return all its segments as separate MiljoeDataClean entries
    /// For MultiLineString features with N segments, returns Vec with N entries, one per segment
    fn parse_miljoedata_feature(feature: Feature, is_avgifter: bool) -> Vec<MiljoeDataClean> {
        let mut results = Vec::new();
        let props = match feature.clone().properties {
            Some(p) => p,
            None => return results,
        };
        let segments = match Self::extract_all_line_segments(&feature) {
            Some(s) => s,
            None => return results,
        };
        let info = if is_avgifter {
            props
                .get("taxa")
                .or_else(|| props.get("value"))
                .or_else(|| props.get("copyvalue"))
                .and_then(|v| v.as_str())
                .unwrap_or("Okänd")
                .to_string()
        } else {
            props
                .get("copy_value")
                .or_else(|| props.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("Okänd")
                .to_string()
        };
        let tid = props
            .get("tid")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
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
        let dag = if is_avgifter {
            0u8
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
        for coordinates in segments {
            results.push(MiljoeDataClean {
                coordinates,
                info: info.clone(),
                tid: tid.clone(),
                dag,
            });
        }
        results
    }
    /// Parse parkering feature and return all segments as ParkeringsDataClean entries
    fn parse_parkering_feature(feature: Feature) -> Vec<ParkeringsDataClean> {
        let mut results = Vec::new();
        let props = match feature.clone().properties {
            Some(p) => p,
            None => return results,
        };

        let segments = match Self::extract_all_line_segments(&feature) {
            Some(s) => s,
            None => return results,
        };

        // Extract parkering-specific fields
        let taxa = props
            .get("taxa")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let antal_platser = props
            .get("antal_platser")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let typ_av_parkering = props
            .get("typ_av_parkering")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        for coordinates in segments {
            results.push(ParkeringsDataClean {
                coordinates,
                taxa: taxa.clone(),
                antal_platser,
                typ_av_parkering: typ_av_parkering.clone(),
            });
        }

        results
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
        for (i, addr) in addresses.iter().take(3).enumerate() {
            println!("  [{}] {} ({:?})", i + 1, addr.adress, addr.postnummer);
        }
        Ok(addresses)
    }
    // Rename to load_miljodata
    pub fn load_miljodata(
        path: &str,
    ) -> Result<Vec<MiljoeDataClean>, Box<dyn std::error::Error>> {
        println!("Loading miljödata from: {}", path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let miljodata: Vec<MiljoeDataClean> = if let GeoJson::FeatureCollection(collection) = geojson {
            collection
                .features
                .into_iter()
                .flat_map(|f| Self::parse_miljoedata_feature(f, false)) // Keep existing logic for miljödata
                .collect()
        } else {
            return Err("Invalid GeoJSON format for miljödata".into());
        };

        println!("Loaded {} miljödata segments", miljodata.len());
        Ok(miljodata)
    }

    // Add new method for parkering
    pub fn load_parkering(
        path: &str,
    ) -> Result<Vec<ParkeringsDataClean>, Box<dyn std::error::Error>> {
        println!("Loading parkeringsavgifter from: {}", path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let parkering: Vec<ParkeringsDataClean> = if let GeoJson::FeatureCollection(collection) = geojson {
            collection
                .features
                .into_iter()
                .flat_map(Self::parse_parkering_feature)
                .collect()
        } else {
            return Err("Invalid GeoJSON format for parkeringsavgifter".into());
        };

        println!("Loaded {} parkering segments", parkering.len());
        Ok(parkering)
    }
}
pub fn api() -> Result<ApiResult, Box<dyn std::error::Error>> {
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_miljodata("data/miljoparkeringar.json")?;
    let parkering = DataLoader::load_parkering("data/parkeringsavgifter.json")?;

    println!("\n✓ Data loading complete");
    println!("  Total addresses: {}", addresses.len());
    println!("  Total miljödata segments: {}", miljodata.len());
    println!("  Total parkering segments: {}", parkering.len());

    Ok((addresses, miljodata, parkering))
}
pub fn api_miljo_only()
-> Result<(Vec<AdressClean>, Vec<MiljoeDataClean>), Box<dyn std::error::Error>> {
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_miljodata("data/miljoparkeringar.json")?;
    Ok((addresses, miljodata))
}
