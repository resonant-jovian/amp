use amp_core::parquet::{ParkingRestriction, read_android_local_addresses};
use std::collections::HashMap;
pub type StaticAddressEntry = ParkingRestriction;
/// Load parquet data from assets
///
/// Searches multiple possible paths for the parking database file.
///
/// # Returns
/// HashMap with keys formatted as "street street_number-postal_code"
pub fn load_parquet_data() -> HashMap<String, StaticAddressEntry> {
    let possible_paths = vec![
        "assets/parking_db.parquet",
        "android/assets/parking_db.parquet",
        "../assets/parking_db.parquet",
        "parking_db.parquet",
    ];
    for path in possible_paths {
        if let Ok(restrictions) = read_android_local_addresses(path) {
            eprintln!("✓ Loaded {} addresses from {}", restrictions.len(), path);
            let mut map = HashMap::new();
            for r in restrictions {
                let key = format!("{} {}-{}", r.street, r.street_number, r.postal_code);
                map.insert(key, r);
            }
            return map;
        }
    }
    eprintln!("⚠️  Warning: Could not load parquet database, using empty dataset");
    eprintln!("   Run: cd server && cargo run --release -- output --android");
    eprintln!("   Then copy .app_addresses.parquet to android/assets/parking_db.parquet",);
    HashMap::new()
}
