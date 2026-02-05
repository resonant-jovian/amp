//! Script to generate debug.parquet file with example addresses
//!
//! Run with: cargo run --bin debug_script
use amp_core::parquet::write_output_parquet;
use amp_core::structs::{OutputData, DB};
use chrono::Datelike;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating debug.parquet with sample addresses...");

    let year = chrono::Utc::now().year();
    let month = chrono::Utc::now().month();

    // Define debug addresses from debug.txt - sample addresses for testing
    let debug_addresses = vec![
        ("Kornettsgatan", "18C", "21150", 1, "0800-1200"),
        ("Claesgatan", "2B", "21426", 2, "0800-1200"),
        ("Östra Kristinelundsvägen", "27D", "21748", 3, "0800-1200"),
        ("Karlskronaplan", "3", "21436", 4, "0800-1200"),
        ("Västra Rönneholmsvägen", "76C", "21741", 5, "0800-1200"),
    ];

    let mut output_entries = Vec::new();

    for (street, number, postal, dag, tid) in debug_addresses {
        println!("\nProcessing: {} {} {} (dag: {})", street, number, postal, dag);

        // Create a DB entry for this address
        if let Some(db) = DB::from_dag_tid(
            Some(postal.to_string()),
            format!("{} {}", street, number),
            Some(street.to_string()),
            Some(number.to_string()),
            Some("Parkering förbjuden".to_string()),
            dag,
            tid,
            Some("Taxa C".to_string()),
            Some(10),
            Some("Längsgående".to_string()),
            year,
            month,
        ) {
            // Convert DB to OutputData for parquet
            let output = OutputData {
                postnummer: db.postnummer.clone(),
                adress: db.adress.clone(),
                gata: db.gata.clone().unwrap_or_default(),
                gatunummer: db.gatunummer.clone().unwrap_or_default(),
                info: db.info.clone(),
                tid: Some(tid.to_string()),
                dag: Some(dag),
                taxa: db.taxa.clone(),
                antal_platser: db.antal_platser,
                typ_av_parkering: db.typ_av_parkering.clone(),
            };
            output_entries.push(output);
            println!("  ✓ Created entry");
        } else {
            println!("  ✗ Failed to create DB entry");
        }
    }

    println!("\n=== Summary ===");
    println!("Successfully created: {} entries", output_entries.len());

    // Write the parquet file with output entries
    if !output_entries.is_empty() {
        let output_path = "android/assets/data/debug.parquet";
        write_output_parquet(output_entries.clone(), output_path)?;
        println!("\n✓ Created {}", output_path);
        println!("  Contains {} debug entries", output_entries.len());
    } else {
        eprintln!("\n⚠ Warning: No entries created, not creating debug.parquet");
    }

    Ok(())
}
