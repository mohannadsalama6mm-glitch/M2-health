use super::*;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
pub const HEADERS: [&str; 7] = [
    "commercial_name_en",
    "commercial_name_ar",
    "scientific_name",
    "manufacturer",
    "drug_class",
    "route",
    "price_egp",
];
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn now() -> String {
    // SQLite supplies the same UTC format used by price history.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}
#[derive(Deserialize)]
struct Config {
    id: String,
    path: String,
    format: String,
    encoding: String,
    delimiter: String,
}
pub fn read(root: &Path) -> Result<(Profile, Vec<Vec<String>>), AppError> {
    let config: Config =
        serde_json::from_slice(&fs::read(root.join("config/catalog-source.json"))?)
            .map_err(|_| invalid("Invalid canonical source configuration."))?;
    if config.id != "egyptian-drugs"
        || config.format != "csv"
        || config.encoding != "utf-8-sig"
        || config.delimiter != ","
    {
        return Err(invalid("Unsupported canonical source configuration."));
    }
    let root = root.canonicalize()?;
    let path = root.join(config.path).canonicalize()?;
    if !path.starts_with(&root) {
        return Err(invalid(
            "Canonical source must remain inside the project resources.",
        ));
    }
    let bytes = fs::read(&path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| invalid("Canonical source must be valid UTF-8."))?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .flexible(false)
        .from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| invalid("Cannot read CSV header."))?;
    if headers.iter().collect::<Vec<_>>() != HEADERS {
        return Err(invalid(
            "CSV columns do not match the canonical seven-column schema.",
        ));
    }
    let mut rows = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|e| invalid(&format!("Structural CSV error: {e}")))?;
        rows.push(row.iter().map(str::to_owned).collect());
    }
    if rows.is_empty() {
        return Err(invalid("Canonical source is empty."));
    }
    Ok((
        Profile {
            path: path.to_string_lossy().into_owned(),
            hash: hash(&bytes),
            size: bytes.len() as u64,
            total_rows: rows.len(),
            columns: HEADERS.iter().map(|s| s.to_string()).collect(),
            profiled_at: now(),
        },
        rows,
    ))
}
