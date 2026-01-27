use std::collections::HashMap;
use amp_core::parquet::{read_android_local_addresses, ParkingRestriction};

pub fn load_parquet_data() -> HashMap<String, ParkingRestriction> {
    let path = "assets/parking_db.parquet";

    match read_android_local_addresses(path) {
        Ok(restrictions) => {
            let mut map = HashMap::new();
            for r in restrictions {
                let key = format!("{} {}-{}", r.gata, r.gatunummer, r.postnummer);
                map.insert(key, r);
            }
            map
        },
        Err(e) => {
            eprintln!("Failed to load parquet: {}", e);
            HashMap::new()
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct StaticAddressEntry {
    pub adress: String,
    pub gata: String,
    pub gatunummer: String,
    pub postnummer: u16,
    pub dag: u8,       // Day of month
    pub tid: String,   // Time interval (e.g., "0800-1000")
    pub info: String,  // Additional info
    pub distance: f64, // For ranking closest matches
}

/// Load static address correlations
/// This is generated from `cargo run --release correlate -c 20 -a kdtree`
/// Run: cargo run --release correlate -c 20 -a kdtree
/// Then parse the output and populate this function
pub fn get_static_addresses() -> HashMap<String, StaticAddressEntry> {
    let mut map = HashMap::new();

    // TODO: Replace with actual correlated data from cargo run --release correlate -c 20 -a kdtree
    // Key format: "gata gatunummer-postnummer"
    let entries = vec![
        StaticAddressEntry {
            adress: "Storgatan 10".to_string(),
            gata: "Storgatan".to_string(),
            gatunummer: "10".to_string(),
            postnummer: 22100,
            dag: 1,
            tid: "0800-1000".to_string(),
            info: "Parkering".to_string(),
            distance: 2.5,
        },
        StaticAddressEntry {
            adress: "Kungsgatan 5".to_string(),
            gata: "Kungsgatan".to_string(),
            gatunummer: "5".to_string(),
            postnummer: 22200,
            dag: 2,
            tid: "0900-1100".to_string(),
            info: "Parkering".to_string(),
            distance: 3.2,
        },
    ];

    for entry in entries {
        let key = format!("{} {}-{}", entry.gata, entry.gatunummer, entry.postnummer);
        map.insert(key, entry);
    }

    map
}

/// Create lookup key from input components
pub fn make_lookup_key(gata: &str, gatunummer: &str, postnummer: &str) -> String {
    format!(
        "{} {}-{}",
        gata.trim(),
        gatunummer.trim(),
        postnummer.trim()
    )
}

/// Lookup address in static correlations
pub fn lookup_address(
    gata: &str,
    gatunummer: &str,
    postnummer: &str,
) -> Option<StaticAddressEntry> {
    let key = make_lookup_key(gata, gatunummer, postnummer);
    get_static_addresses().get(&key).cloned()
}
