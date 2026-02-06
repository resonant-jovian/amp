//! GeoJSON data loading for Swedish parking and address information.
//!
//! This module provides functionality to load and parse GeoJSON files containing:
//! - Address data with coordinates and postal codes
//! - Environmental parking restrictions (street cleaning schedules)
//! - Parking zone information with pricing tiers
//!
//! The GeoJSON files typically come from Swedish municipal open data sources
//! (e.g., Malmö stad) and contain both Point geometries (addresses) and
//! LineString/MultiLineString geometries (parking zones).
//!
//! # Examples
//!
//! ## Loading All Data Sources
//!
//! ```no_run
//! use amp_core::api::api;
//!
//! let (addresses, miljoe_data, parkering_data) = api()?;
//! println!("Loaded {} addresses", addresses.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Loading Individual Datasets
//!
//! ```no_run
//! use amp_core::api::DataLoader;
//!
//! let addresses = DataLoader::load_addresses("data/adresser.json")?;
//! let parking_zones = DataLoader::load_parkering("data/parkeringsavgifter.json")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::structs::{AdressClean, MiljoeDataClean, ParkeringsDataClean};
use geojson::{Feature, GeoJson};
use rust_decimal::Decimal;
use std::fs;

/// Tuple containing all three data types loaded from GeoJSON sources.
///
/// The elements are:
/// 1. Addresses with coordinates ([`AdressClean`])
/// 2. Environmental parking restrictions ([`MiljoeDataClean`])
/// 3. Parking zones with pricing ([`ParkeringsDataClean`])
pub type ApiResult = (
    Vec<AdressClean>,
    Vec<MiljoeDataClean>,
    Vec<ParkeringsDataClean>,
);

/// Utility for loading and parsing GeoJSON data files.
///
/// This struct provides static methods for loading different types of parking
/// and address data from GeoJSON files. It handles both Point geometries
/// (for addresses) and LineString/MultiLineString geometries (for parking zones).
///
/// # Implementation Notes
///
/// - LineString and MultiLineString features are split into individual segments
/// - Each segment becomes a separate data entry for efficient spatial matching
/// - Coordinate conversion uses [`rust_decimal::Decimal`] for precision
/// - Failed conversions are logged but don't stop the loading process
pub struct DataLoader;

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl DataLoader {
    /// Create a new DataLoader instance.
    ///
    /// Since all methods are static, this is primarily for API consistency.
    pub fn new() -> Self {
        Self
    }

    /// Extract point coordinates from a GeoJSON Point feature.
    ///
    /// # Arguments
    ///
    /// * `feature` - GeoJSON feature containing Point geometry
    ///
    /// # Returns
    ///
    /// `Some([longitude, latitude])` if extraction succeeds, `None` otherwise.
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

    /// Extract all line segments from LineString or MultiLineString geometry.
    ///
    /// This function splits MultiLineString features into individual segments,
    /// with each segment represented as `[[start_x, start_y], [end_x, end_y]]`.
    /// This allows efficient spatial correlation by treating each street segment
    /// independently.
    ///
    /// # Arguments
    ///
    /// * `feature` - GeoJSON feature with LineString or MultiLineString geometry
    ///
    /// # Returns
    ///
    /// Vector of segment coordinate pairs, or `None` if extraction fails.
    ///
    /// # Implementation Details
    ///
    /// - Uses `.first()` and `.last()` for safe array access
    /// - Invalid segments are logged and skipped, not causing failure
    /// - Returns `None` only if no valid segments are found
    fn extract_all_line_segments(feature: &Feature) -> Option<Vec<[[Decimal; 2]; 2]>> {
        let mut segments = Vec::new();

        if let Some(ref geom) = feature.geometry {
            match &geom.value {
                geojson::Value::LineString(coords) => {
                    let first = coords.first()?;
                    let last = coords.last()?;

                    if first.len() < 2 || last.len() < 2 {
                        eprintln!("[API] Invalid coordinate length in LineString");
                        return None;
                    }

                    let x1 = Decimal::try_from(first[0]).ok()?;
                    let y1 = Decimal::try_from(first[1]).ok()?;
                    let x2 = Decimal::try_from(last[0]).ok()?;
                    let y2 = Decimal::try_from(last[1]).ok()?;

                    segments.push([[x1, y1], [x2, y2]]);
                }
                geojson::Value::MultiLineString(lines) => {
                    for line in lines {
                        let first = match line.first() {
                            Some(f) => f,
                            None => {
                                eprintln!("[API] Empty line in MultiLineString");
                                continue;
                            }
                        };
                        let last = match line.last() {
                            Some(l) => l,
                            None => {
                                eprintln!("[API] Empty line in MultiLineString");
                                continue;
                            }
                        };

                        if first.len() < 2 || last.len() < 2 {
                            eprintln!("[API] Invalid coordinate length in MultiLineString",);
                            continue;
                        }

                        let x1 = match Decimal::try_from(first[0]) {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("[API] Failed to convert x1 coordinate");
                                continue;
                            }
                        };
                        let y1 = match Decimal::try_from(first[1]) {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("[API] Failed to convert y1 coordinate");
                                continue;
                            }
                        };
                        let x2 = match Decimal::try_from(last[0]) {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("[API] Failed to convert x2 coordinate");
                                continue;
                            }
                        };
                        let y2 = match Decimal::try_from(last[1]) {
                            Ok(v) => v,
                            Err(_) => {
                                eprintln!("[API] Failed to convert y2 coordinate");
                                continue;
                            }
                        };

                        segments.push([[x1, y1], [x2, y2]]);
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

    /// Legacy function for point coordinate extraction.
    ///
    /// Kept for backwards compatibility but not used in current code.
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

    /// Parse a GeoJSON address feature into [`AdressClean`].
    ///
    /// # Expected GeoJSON Properties
    ///
    /// - `POSTNR`: Postal code (optional)
    /// - `BELADRESS`: Full address string (required)
    /// - `ADRESSOMR`: Street name (required)
    /// - `ADRESSPLAT`: Street number (required)
    ///
    /// # Returns
    ///
    /// `Some(AdressClean)` if all required fields are present, `None` otherwise.
    fn parse_address_feature(feature: Feature) -> Option<AdressClean> {
        let props = feature.clone().properties?;
        let coordinates = Self::extract_point_coordinates(&feature)?;

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

    /// Parse environmental parking restriction feature into multiple [`MiljoeDataClean`] entries.
    ///
    /// For MultiLineString features with N segments, this returns N separate entries,
    /// one per segment. This enables per-segment spatial matching.
    ///
    /// # Arguments
    ///
    /// * `feature` - GeoJSON feature with parking restriction data
    /// * `is_avgifter` - `true` for parking fees, `false` for environmental restrictions
    ///
    /// # Expected Properties
    ///
    /// - `taxa`, `value`, or `copyvalue`: Zone information
    /// - `tid`: Time range string
    /// - `day`: Day of month (for environmental restrictions only)
    ///
    /// # Returns
    ///
    /// Vector of `MiljoeDataClean` entries, one per line segment.
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

        let tid = props.get("tid").unwrap().to_string();

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

    /// Parse parking zone feature into multiple [`ParkeringsDataClean`] entries.
    ///
    /// Similar to [`parse_miljoedata_feature`], this splits MultiLineString features
    /// into individual segments.
    ///
    /// # Expected Properties
    ///
    /// - `taxa`: Parking zone tier (e.g., "Taxa A", "Taxa B")
    /// - `antal_platser`: Number of parking spots
    /// - `typ_av_parkering`: Parking type (e.g., "Längsgående 6")
    ///
    /// # Returns
    ///
    /// Vector of `ParkeringsDataClean` entries, one per line segment.
    ///
    /// [`parse_miljoedata_feature`]: Self::parse_miljoedata_feature
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

    /// Load address data from a GeoJSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GeoJSON file containing address data
    ///
    /// # Returns
    ///
    /// Vector of [`AdressClean`] entries with coordinates and postal codes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file is not valid GeoJSON
    /// - The GeoJSON is not a FeatureCollection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use amp_core::api::DataLoader;
    ///
    /// let addresses = DataLoader::load_addresses("data/adresser.json")?;
    /// println!("Loaded {} addresses", addresses.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Load environmental parking restriction data from a GeoJSON file.
    ///
    /// This typically contains street cleaning schedules and time-restricted zones.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GeoJSON file containing environmental parking data
    ///
    /// # Returns
    ///
    /// Vector of [`MiljoeDataClean`] entries, with MultiLineString features split
    /// into individual segments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file is not valid GeoJSON
    /// - The GeoJSON is not a FeatureCollection
    pub fn load_miljodata(path: &str) -> Result<Vec<MiljoeDataClean>, Box<dyn std::error::Error>> {
        println!("Loading miljödata from: {}", path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let miljodata: Vec<MiljoeDataClean> =
            if let GeoJson::FeatureCollection(collection) = geojson {
                collection
                    .features
                    .into_iter()
                    .flat_map(|f| Self::parse_miljoedata_feature(f, false))
                    .collect()
            } else {
                return Err("Invalid GeoJSON format for miljödata".into());
            };

        println!("Loaded {} miljödata segments", miljodata.len());
        Ok(miljodata)
    }

    /// Load parking zone data from a GeoJSON file.
    ///
    /// This contains paid parking zones with pricing tiers (taxa) and capacity information.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GeoJSON file containing parking zone data
    ///
    /// # Returns
    ///
    /// Vector of [`ParkeringsDataClean`] entries, with MultiLineString features split
    /// into individual segments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file is not valid GeoJSON
    /// - The GeoJSON is not a FeatureCollection
    pub fn load_parkering(
        path: &str,
    ) -> Result<Vec<ParkeringsDataClean>, Box<dyn std::error::Error>> {
        println!("Loading parkeringsavgifter from: {}", path);
        let content = fs::read_to_string(path)?;
        let geojson: GeoJson = content.parse()?;

        let parkering: Vec<ParkeringsDataClean> =
            if let GeoJson::FeatureCollection(collection) = geojson {
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

/// Load all three data sources from standard file paths.
///
/// This is a convenience function that loads addresses, environmental parking
/// restrictions, and parking zones from the default `data/` directory.
///
/// # Expected File Paths
///
/// - `data/adresser.json` - Address data
/// - `data/miljoparkeringar.json` - Environmental parking restrictions
/// - `data/parkeringsavgifter.json` - Parking zone pricing
///
/// # Returns
///
/// Tuple of `(addresses, miljoe_data, parkering_data)` if all files load successfully.
///
/// # Errors
///
/// Returns an error if any file cannot be loaded or parsed.
///
/// # Examples
///
/// ```no_run
/// use amp_core::api::api;
///
/// let (addresses, miljoe, parkering) = api()?;
/// println!("Loaded {} addresses", addresses.len());
/// println!("Loaded {} miljoe segments", miljoe.len());
/// println!("Loaded {} parkering segments", parkering.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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

/// Load only addresses and environmental parking data (excludes parking zones).
///
/// This is useful for testing or when parking zone pricing is not needed.
///
/// # Returns
///
/// Tuple of `(addresses, miljoe_data)` if both files load successfully.
///
/// # Errors
///
/// Returns an error if either file cannot be loaded or parsed.
pub fn api_miljo_only(
) -> Result<(Vec<AdressClean>, Vec<MiljoeDataClean>), Box<dyn std::error::Error>> {
    let addresses = DataLoader::load_addresses("data/adresser.json")?;
    let miljodata = DataLoader::load_miljodata("data/miljoparkeringar.json")?;
    Ok((addresses, miljodata))
}
