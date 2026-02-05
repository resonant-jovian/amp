//! Creates a debug.parquet file with ONLY the address strings from debug.txt
//! All other fields are left as NULL, mimicking user input via the "Add Address" button
//!
//! Format of debug.txt (CSV):
//! postnummer,full_address,street,number (comment)
//! Example: 211 50,Kornettsgatan 18C,Kornettsgatan,18C (dag 1)
use parquet::basic::Compression;
use parquet::column::writer::ColumnWriter;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("scripts/debug.txt")?;
    let reader = BufReader::new(file);
    let mut addresses = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            let full_address = parts[1].trim();
            addresses.push(full_address.to_string());
        }
    }
    println!("Loaded {} addresses from debug.txt", addresses.len());
    let schema = Arc::new(parse_message_type(
        "
        message schema {
            OPTIONAL BYTE_ARRAY adress (UTF8);
        }
        ",
    )?);
    let file = File::create("android/app/src/main/assets/debug.parquet")?;
    let props = Arc::new(
        WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build(),
    );
    let mut writer = SerializedFileWriter::new(file, schema.clone(), props)?;
    for address in &addresses {
        let mut row_group_writer = writer.next_row_group()?;
        if let Some(mut col_writer) = row_group_writer.next_column()? {
            if let ColumnWriter::ByteArrayColumnWriter(ref mut typed_writer) = col_writer {
                let byte_array = parquet::data_type::ByteArray::from(address.as_str());
                typed_writer.write_batch(&[byte_array], Some(&[1]), None)?;
                row_group_writer.close_column(col_writer)?;
            }
        }
        writer.close_row_group(row_group_writer)?;
    }
    writer.close()?;
    println!(
        "✅ Created debug.parquet with {} address entries",
        addresses.len()
    );
    println!("   Each entry contains ONLY the address field");
    println!("   Location: android/app/src/main/assets/debug.parquet");
    Ok(())
}
