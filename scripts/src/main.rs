//! Creates a debug.parquet file with ONLY the address strings from debug.txt
//! All other fields are left as NULL, mimicking user input via the "Add Address" button
//!
//! Format of debug.txt (CSV):
//! postnummer,full_address,street,number (comment)
//! Example: 211 50,Kornettsgatan 18C,Kornettsgatan,18C (dag 1)
//!
//! The script reads the full_address column (index 1) and creates a minimal parquet
//! with a single "adress" field. When loaded in the app, StoredAddress::new() will
//! perform fuzzy matching against the static parking database.

use arrow::array::StringBuilder;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::{EnabledStatistics, WriterProperties};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

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
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse CSV: postnummer,full_address,street,number (comment)
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 2 {
            eprintln!("⚠️  Line {}: Invalid format (expected at least 2 CSV columns)", line_num);
            continue;
        }
        
        // Extract full address (column 1)
        let full_address = parts[1].trim();
        
        if full_address.is_empty() {
            eprintln!("⚠️  Line {}: Empty address", line_num);
            continue;
        }
        
        println!("   [{}] {}", addresses.len(), full_address);
        addresses.push(full_address.to_string());
    }
    
    println!();
    println!("✅ Loaded {} addresses from debug.txt", addresses.len());
    println!();
    
    if addresses.is_empty() {
        return Err("No addresses found in debug.txt".into());
    }
    
    // Create minimal schema with ONLY the address field
    let schema = Arc::new(Schema::new(vec![Field::new(
        "adress",
        DataType::Utf8,
        false, // NOT NULL - every entry must have an address
    )]));
    
    let file = File::create("android/assets/data/debug.parquet")?;
    let props = WriterProperties::builder()
        .set_statistics_enabled(EnabledStatistics::None)
        .build();
    
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;
    
    // Build address column
    let mut address_builder = StringBuilder::new();
    for address in &addresses {
        address_builder.append_value(address);
    }
    
    // Create single-column batch
    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(address_builder.finish())])?;
    
    writer.write(&batch)?;
    writer.close()?;
    
    println!("✅ Created debug.parquet with {} address entries", addresses.len());
    println!("   Schema: [adress: String NOT NULL]");
    println!("   All other LocalData fields are NULL/missing");
    println!();
    println!("📍 Location: android/assets/data/debug.parquet");
    println!();
    println!("💡 When loaded, each address will be processed via StoredAddress::new()");
    println!("   which performs fuzzy matching against the static parking database.");
    println!("   This mimics clicking 'Add Address' button {} times.", addresses.len());
    println!();
    
    Ok(())
}
