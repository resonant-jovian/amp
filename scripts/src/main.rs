//! Script to generate debug.parquet file with example addresses
//!
//! Run with: cargo run --bin generate_debug_parquet
use amp_core::correlation_algorithms::{ParkeringCorrelationAlgo, RTreeSpatialParkeringAlgo};
use amp_core::parquet::{build_db_parquet, read_addresses, read_parkering};
use amp_core::structs::{AdressClean, DB, OutputData};
use chrono::Datelike;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating debug.parquet with correlation matching...");

    // Load actual data
    println!("Loading parkering data...");
    let parkering_data = read_parkering("data/parkering.parquet")?;
    println!("Loaded {} parkering entries", parkering_data.len());

    println!("Loading address data...");
    let address_data = read_addresses("data/adresser.parquet")?;
    println!("Loaded {} addresses", address_data.len());

    // Initialize correlation algorithm (same as UI uses)
    let algo = RTreeSpatialParkeringAlgo::new(&parkering_data);

    let year = chrono::Utc::now().year();
    let month = chrono::Utc::now().month();

    // Define debug addresses from debug.txt - all 33 addresses
    let debug_addresses = vec![
        ("Kornettsgatan", "18C", "21150", 1),
        ("Claesgatan", "2B", "21426", 2),
        ("Östra Kristinelundsvägen", "27D", "21748", 3),
        ("Karlskronaplan", "3", "21436", 4),
        ("Västra Rönneholmsvägen", "76C", "21741", 5),
        ("Vitemöllegatan", "11A", "21442", 6),
        ("Docentgatan", "1B", "21552", 7),
        ("Eriksfältsgatan", "98B", "21550", 8),
        ("Lantmannagatan", "50 U1", "21448", 9),
        ("Pysslinggatan", "4", "21238", 10),
        ("Celsiusgatan", "13A U1", "21214", 11),
        ("Kapellgatan", "14 U4", "21421", 12),
        ("Tegnérgatan", "25B", "21614", 13),
        ("S:t Pauli kyrkogata", "13B", "21149", 14),
        ("Östra Stallmästaregatan", "18B", "21749", 15),
        ("Södervärnsgatan", "9B U1", "21427", 16),
        ("Carl Hillsgatan", "10B", "21756", 17),
        ("Köpenhamnsvägen", "46A", "21771", 18),
        ("Bangatan", "13", "21426", 19),
        ("Smålandsgatan", "20A", "21430", 20),
        ("Tycho Brahegatan", "26", "21612", 21),
        ("Storgatan", "43K", "21142", 22),
        ("Östergårdsgatan", "1 U13", "21222", 23),
        ("Byggmästaregatan", "5", "21130", 24),
        ("Lantmannagatan", "11A", "21444", 25),
        ("Zenithgatan", "42C", "21214", 26),
        ("Bragegatan", "37B", "21446", 27),
        ("Idunsgatan", "67B", "21446", 28),
        ("Värnhemsgatan", "2A", "21215", 29),
        ("Sånekullavägen", "36A", "21774", 30),
        ("Amiralsgatan", "83E", "21437", 31), // ingen städning men parkering
        ("Docentgatan", "3A", "21552", 32),    // städning men ingen parkerings avgift
        ("Låssasgatan", "11A", "11111", 33),   // false street
    ];

    let mut matched_entries = Vec::new();
    let mut correlation_failures = Vec::new();

    for (street, number, postal, expected_dag) in debug_addresses {
        // Find the address in the address data
        let address = address_data.iter().find(|addr| {
            addr.gata.as_deref() == Some(street)
                && addr.gatunummer.as_deref() == Some(number)
                && addr.postnummer.as_deref() == Some(postal)
        });

        if let Some(addr) = address {
            println!("\nProcessing: {} {} {}", street, number, postal);

            // Perform correlation (same as UI does)
            match algo.correlate(addr, &parkering_data) {
                Some((idx, distance)) => {
                    let parkering = &parkering_data[idx];
                    println!(
                        "  ✓ Matched with distance {:.2}m: {} (dag: {:?})",
                        distance, parkering.adress, parkering.dag
                    );

                    // Create DB entry from correlation
                    if let Some(db) = DB::from_dag_tid(
                        parkering.postnummer.clone(),
                        parkering.adress.clone(),
                        Some(parkering.gata.clone()),
                        Some(parkering.gatunummer.clone()),
                        parkering.info.clone(),
                        parkering.dag,
                        &parkering.tid,
                        parkering.taxa.clone(),
                        parkering.antal_platser,
                        parkering.typ_av_parkering.clone(),
                        year,
                        month,
                    ) {
                        // Convert DB to OutputData for parquet
                        let output = OutputData {
                            postnummer: db.postnummer.clone(),
                            adress: db.adress.clone(),
                            gata: db.gata.clone(),
                            gatunummer: db.gatunummer.clone(),
                            info: db.info.clone(),
                            tid: db.tid.clone(),
                            dag: db.dag,
                            taxa: db.taxa.clone(),
                            antal_platser: db.antal_platser,
                            typ_av_parkering: db.typ_av_parkering.clone(),
                        };
                        matched_entries.push(output);
                    }
                }
                None => {
                    println!("  ✗ No correlation match found (expected dag: {})", expected_dag);
                    correlation_failures.push((street, number, postal, expected_dag));
                }
            }
        } else {
            println!("\n✗ Address not found in database: {} {} {}", street, number, postal);
            correlation_failures.push((street, number, postal, expected_dag));
        }
    }

    println!("\n=== Summary ===");
    println!("Successfully matched: {} addresses", matched_entries.len());
    println!("Failed correlations: {} addresses", correlation_failures.len());

    if !correlation_failures.is_empty() {
        println!("\nFailed addresses:");
        for (street, number, postal, dag) in correlation_failures {
            println!("  - {} {} {} (expected dag: {})", street, number, postal, dag);
        }
    }

    // Write the parquet file with matched entries
    if !matched_entries.is_empty() {
        let buffer = build_db_parquet(matched_entries)?;
        fs::write("android/assets/data/debug.parquet", &buffer)?;
        println!(
            "\n✓ Created android/assets/data/debug.parquet ({} bytes)",
            buffer.len()
        );
        println!("  Contains {} valid correlation matches", matched_entries.len());
    } else {
        eprintln!("\n⚠ Warning: No matches found, not creating debug.parquet");
    }

    Ok(())
}
