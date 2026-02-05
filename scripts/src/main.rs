//! Creates a debug.parquet file with address and postal code from debug.txt
//! Other fields are left as NULL, mimicking user input via the "Add Address" button
//!
//! Format of debug.txt (CSV):
//! postnummer,full_address,street,number (comment)
//! Example: 211 50,Kornettsgatan 18C,Kornettsgatan,18C (dag 1)
//!
//! The script reads both postnummer (column 0) and full_address (column 1).
//! Creates a minimal parquet with 'adress' and 'postnummer' fields.
//! When loaded in the app, StoredAddress::new() will perform fuzzy matching
//! against the static parking database using both fields.
use arrow::array::StringBuilder;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::{EnabledStatistics, WriterProperties};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
#[derive(Debug)]
struct DebugAddress {
    postnummer: String,
    adress: String,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Creating debug.parquet from debug.txt...");
    println!();
    let file = File::open("scripts/debug.txt")?;
    let reader = BufReader::new(file);
    let mut addresses = Vec::new();
    let mut line_num = 0;
    for line in reader.lines() {
        line_num += 1;
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            eprintln!(
                "⚠️  Line {}: Invalid format (expected at least 2 CSV columns)",
                line_num,
            );
            continue;
        }
        let postnummer_raw = parts[0].trim();
        let full_address = parts[1].trim();
        if full_address.is_empty() {
            eprintln!("⚠️  Line {}: Empty address", line_num);
            continue;
        }
        let postnummer = postnummer_raw.replace(' ', "");
        if postnummer.is_empty() || !postnummer.chars().all(|c| c.is_ascii_digit()) {
            eprintln!(
                "⚠️  Line {}: Invalid postal code '{}'",
                line_num, postnummer_raw,
            );
            continue;
        }
        println!("   [{}] {} ({})", addresses.len(), full_address, postnummer);
        addresses.push(DebugAddress {
            postnummer,
            adress: full_address.to_string(),
        });
    }
    println!();
    println!("✅ Loaded {} addresses from debug.txt", addresses.len());
    println!();
    if addresses.is_empty() {
        return Err("No addresses found in debug.txt".into());
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("adress", DataType::Utf8, false),
        Field::new("postnummer", DataType::Utf8, false),
    ]));
    let file = File::create("android/assets/data/debug.parquet")?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;
    let mut address_builder = StringBuilder::new();
    let mut postnummer_builder = StringBuilder::new();
    for entry in &addresses {
        address_builder.append_value(&entry.adress);
        postnummer_builder.append_value(&entry.postnummer);
    }
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(address_builder.finish()),
            Arc::new(postnummer_builder.finish()),
        ],
    )?;
    writer.write(&batch)?;
    writer.close()?;
    println!(
        "✅ Created debug.parquet with {} address entries",
        addresses.len()
    );
    println!("   Schema: [adress: String NOT NULL, postnummer: String NOT NULL]");
    println!("   All other LocalData fields are NULL/missing");
    println!();
    println!("📍 Location: android/assets/data/debug.parquet");
    println!();
    println!("💡 When loaded, each address will be processed via StoredAddress::new()",);
    println!("   with street, number, and postal code for accurate fuzzy matching.");
    println!(
        "   This mimics clicking 'Add Address' button {} times.",
        addresses.len()
    );
    println!();
    Ok(())
}
