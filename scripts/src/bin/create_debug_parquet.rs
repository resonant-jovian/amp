//! Creates a debug.parquet file with ONLY the address strings from debug.txt
//! All other fields are left as NULL, mimicking user input via the "Add Address" button
//!
//! Format of debug.txt (CSV):
//! postnummer,full_address,street,number (comment)
//! Example: 211 50,Kornettsgatan 18C,Kornettsgatan,18C (dag 1)

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use arrow::array::StringBuilder;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::{EnabledStatistics, WriterProperties};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read debug.txt
    let file = File::open("scripts/debug.txt")?;
    let reader = BufReader::new(file);

    let mut addresses = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines or comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse CSV: postnummer,full_address,street,number (comment)
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let full_address = parts[1].trim();
            addresses.push(full_address.to_string());
        }
    }

    println!("Loaded {} addresses from debug.txt", addresses.len());

    // Define schema - ONLY adress field
    let schema = Arc::new(Schema::new(vec![
        Field::new("adress", DataType::Utf8, false),
    ]));

    // Create parquet file with Arrow writer
    let file = File::create("android/app/src/main/assets/debug.parquet")?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;

    // Build the address column
    let mut address_builder = StringBuilder::new();
    for address in &addresses {
        address_builder.append_value(address);
    }

    // Create record batch
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(address_builder.finish())],
    )?;

    // Write batch and close
    writer.write(&batch)?;
    writer.close()?;

    println!("✅ Created debug.parquet with {} address entries", addresses.len());
    println!("   Each entry contains ONLY the address field");
    println!("   Location: android/app/src/main/assets/debug.parquet");

    Ok(())
}
