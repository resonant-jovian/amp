//! Debug script to generate debug.parquet with real correlation matching
//!
//! This script:
//! 1. Loads all 33 addresses from the hardcoded debug.txt data
//! 2. Uses the actual KDTree correlation algorithm (same as local.parquet generation)
//! 3. Correlates addresses with miljödata and parkeringsavgifter
//! 4. Writes results to android/assets/data/debug.parquet using LocalData schema
//! 5. Sets active=true and valid=true (when miljödata exists)
//!
//! Run with: cargo run --bin debug_script
use amp_core::api::api;
use amp_core::correlation_algorithms::{
    CorrelationAlgo, KDTreeParkeringAlgo, KDTreeSpatialAlgo, ParkeringCorrelationAlgo,
};
use amp_core::parquet::build_local_parquet;
use amp_core::structs::{AdressClean, LocalData};
use rust_decimal::Decimal;
use std::fs;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Debug Script - Generating debug.parquet with real matching\n");
    println!("📂 Loading data from JSON files...");
    let (all_addresses, miljodata, parkering) = api()?;
    println!("  ✓ Loaded {} addresses", all_addresses.len());
    println!("  ✓ Loaded {} miljödata zones", miljodata.len());
    println!("  ✓ Loaded {} parkering zones", parkering.len());
    let debug_specs = vec![
        ("211 50", "Kornettsgatan 18C", "Kornettsgatan", "18C"),
        ("214 26", "Claesgatan 2B", "Claesgatan", "2B"),
        (
            "217 48",
            "Östra Kristinelundsvägen 27D",
            "Östra Kristinelundsvägen",
            "27D",
        ),
        ("214 36", "Karlskronaplan 3", "Karlskronaplan", "3"),
        (
            "217 41",
            "Västra Rönneholmsvägen 76C",
            "Västra Rönneholmsvägen",
            "76C",
        ),
        ("214 42", "Vitemöllegatan 11A", "Vitemöllegatan", "11A"),
        ("215 52", "Docentgatan 1B", "Docentgatan", "1B"),
        ("215 50", "Eriksfältsgatan 98B", "Eriksfältsgatan", "98B"),
        ("214 48", "Lantmannagatan 50 U1", "Lantmannagatan", "50 U1"),
        ("212 38", "Pysslinggatan 4", "Pysslinggatan", "4"),
        ("212 14", "Celsiusgatan 13A U1", "Celsiusgatan", "13A U1"),
        ("214 21", "Kapellgatan 14 U4", "Kapellgatan", "14 U4"),
        ("216 14", "Tegnérgatan 25B", "Tegnérgatan", "25B"),
        (
            "211 49",
            "S:t Pauli kyrkogata 13B",
            "S:t Pauli kyrkogata",
            "13B",
        ),
        (
            "217 49",
            "Östra Stallmästaregatan 18B",
            "Östra Stallmästaregatan",
            "18B",
        ),
        (
            "214 27",
            "Södervärnsgatan 9B U1",
            "Södervärnsgatan",
            "9B U1",
        ),
        ("217 56", "Carl Hillsgatan 10B", "Carl Hillsgatan", "10B"),
        ("217 71", "Köpenhamnsvägen 46A", "Köpenhamnsvägen", "46A"),
        ("214 26", "Bangatan 13", "Bangatan", "13"),
        ("214 30", "Smålandsgatan 20A", "Smålandsgatan", "20A"),
        ("216 12", "Tycho Brahegatan 26", "Tycho Brahegatan", "26"),
        ("211 42", "Storgatan 43K", "Storgatan", "43K"),
        (
            "212 22",
            "Östergårdsgatan 1 U13",
            "Östergårdsgatan",
            "1 U13",
        ),
        ("211 30", "Byggmästaregatan 5", "Byggmästaregatan", "5"),
        ("214 44", "Lantmannagatan 11A", "Lantmannagatan", "11A"),
        ("212 14", "Zenithgatan 42C", "Zenithgatan", "42C"),
        ("214 46", "Bragegatan 37B", "Bragegatan", "37B"),
        ("214 46", "Idunsgatan 67B", "Idunsgatan", "67B"),
        ("212 15", "Värnhemsgatan 2A", "Värnhemsgatan", "2A"),
        ("217 74", "Sånekullavägen 36A", "Sånekullavägen", "36A"),
        ("214 37", "Amiralsgatan 83E", "Amiralsgatan", "83E"),
        ("215 52", "Docentgatan 3A", "Docentgatan", "3A"),
        ("111 11", "Låssasgatan 11A", "Låssasgatan", "11A"),
    ];
    println!("\n🔍 Finding addresses in dataset...");
    let mut debug_addresses = Vec::new();
    for (postnummer, full_addr, gata, gatunummer) in debug_specs {
        let found = all_addresses.iter().find(|addr| {
            addr.adress == full_addr
                && addr.postnummer.as_ref().map(|p| p.replace(" ", ""))
                    == Some(postnummer.replace(" ", ""))
        });
        if let Some(addr) = found {
            debug_addresses.push(addr.clone());
            println!("  ✓ Found: {}", full_addr);
        } else {
            println!(
                "  ⚠ Not found in dataset: {} - creating placeholder",
                full_addr
            );
            debug_addresses.push(AdressClean {
                coordinates: [Decimal::ZERO, Decimal::ZERO],
                postnummer: Some(postnummer.to_string()),
                adress: full_addr.to_string(),
                gata: gata.to_string(),
                gatunummer: gatunummer.to_string(),
            });
        }
    }
    println!(
        "\n📊 Successfully prepared {} debug addresses",
        debug_addresses.len()
    );
    println!("\n🔄 Running correlation with KDTree algorithm...");
    let miljo_algo = KDTreeSpatialAlgo::new(&miljodata);
    let parkering_algo = KDTreeParkeringAlgo::new(&parkering);
    let cutoff = 20.0;
    let mut local_entries = Vec::new();
    let mut matched_count = 0;
    for addr in &debug_addresses {
        let miljo_match = miljo_algo.correlate(addr, &miljodata);
        let (miljo_info, miljo_tid, miljo_dag, has_miljo) = if let Some((idx, dist)) = miljo_match {
            if dist <= cutoff {
                if let Some(miljo_data) = miljodata.get(idx) {
                    matched_count += 1;
                    (
                        Some(miljo_data.info.clone()),
                        Some(miljo_data.tid.clone()),
                        Some(miljo_data.dag),
                        true,
                    )
                } else {
                    (None, None, None, false)
                }
            } else {
                (None, None, None, false)
            }
        } else {
            (None, None, None, false)
        };
        let parkering_match = parkering_algo.correlate(addr, &parkering);
        let (taxa, antal_platser, typ_av_parkering) = if let Some((idx, dist)) = parkering_match {
            if dist <= cutoff {
                if let Some(p_data) = parkering.get(idx) {
                    (
                        Some(p_data.taxa.clone()),
                        Some(p_data.antal_platser),
                        Some(p_data.typ_av_parkering.clone()),
                    )
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };
        let local_data = LocalData {
            valid: has_miljo,
            active: true,
            postnummer: addr.postnummer.clone(),
            adress: addr.adress.clone(),
            gata: Some(addr.gata.clone()),
            gatunummer: Some(addr.gatunummer.clone()),
            info: miljo_info,
            tid: miljo_tid,
            dag: miljo_dag,
            taxa,
            antal_platser,
            typ_av_parkering,
        };
        local_entries.push(local_data);
    }
    println!("  ✓ Correlation complete");
    println!(
        "  ✓ Matched: {}/{} addresses",
        matched_count,
        debug_addresses.len()
    );
    let output_path = "android/assets/data/debug.parquet";
    println!("\n💾 Writing to {} using LocalData schema...", output_path);
    let parquet_bytes = build_local_parquet(local_entries.clone())?;
    fs::write(output_path, parquet_bytes)?;
    println!("\n✅ Debug script complete!");
    println!(
        "  ✓ Created {} with {} entries",
        output_path,
        local_entries.len()
    );
    println!(
        "  ✓ {} addresses have miljödata matches (valid=true)",
        matched_count
    );
    println!("  ✓ All addresses set to active=true by default");
    println!("  ✓ Schema: LocalData (compatible with Android debug mode)");
    Ok(())
}
