// Correlation logic has been moved to server/src/main.rs
// This module is deprecated and kept for compatibility only.
//
// The new correlation flow:
// 1. Load addresses from api.rs
// 2. Load both miljödata and parkering datasets from api.rs
// 3. Correlate each dataset separately using correlation_algorithms
// 4. Merge results in server/src/main.rs
// 5. Write to parquet using parquet.rs
