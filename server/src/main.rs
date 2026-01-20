use amp_core::api::api;
use amp_core::correlation::correlation;
use amp_core::structs::{AdressClean, MiljoeDataClean};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch data from ArcGIS services
    let (addresses, zones): (Vec<AdressClean>, Vec<MiljoeDataClean>) = api().await?;
    println!("Loaded {} addresses, {} zones", addresses.len(), zones.len());

    // 2. Correlate addresses to parking zones
    let results = correlation(addresses, zones);

    // 3. Filter for relevant matches
    let matched: Vec<_> = results
        .iter()
        .filter(|r| r.relevant)
        .collect();

    println!("Found {} matching addresses", matched.len());
    for result in &matched {
        println!("- {}: {} ({})",
                 result.adress,
                 result.info,
                 result.relevant,
        );
    }

    Ok(())
}