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
    #[serde(default)]
    import_policy: Option<String>,
    id: String,
    path: String,
    format: String,
    encoding: String,
    delimiter: String,
    #[serde(default)]
    current_version: Option<String>,
    #[serde(default)]
    versions: Vec<SourceVersion>,
}
/// Strict field-boundary quoting check. No repair or whitespace stripping of values.
fn quoting_issue(bytes: &[u8]) -> bool {
    let mut state = 0u8; // field start, unquoted, quoted, after closing quote
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match state {
            0 => match b {
                b'"' => state = 2,
                b',' | b'\r' | b'\n' => {}
                _ => state = 1,
            },
            1 => match b {
                b'"' => return true,
                b',' | b'\r' | b'\n' => state = 0,
                _ => {}
            },
            2 if b == b'"' => {
                if bytes.get(i + 1) == Some(&b'"') {
                    i += 1;
                } else {
                    state = 3;
                }
            }
            3 => match b {
                b',' | b'\r' | b'\n' => state = 0,
                _ => return true,
            },
            _ => {}
        }
        i += 1;
    }
    state == 2
}
#[derive(Deserialize)]
struct SourceVersion {
    version: String,
    sha256: String,
    size_bytes: Option<u64>,
    data_rows: usize,
    apply_allowed: bool,
    #[serde(default)]
    trim_headers: bool,
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
    let source_hash = hash(&bytes);
    let version = match &config.current_version {
        Some(current) => Some(
            config
                .versions
                .iter()
                .find(|v| &v.version == current)
                .ok_or_else(|| invalid("Current canonical source version is not registered."))?,
        ),
        None if config.versions.is_empty() => None,
        None => return Err(invalid("Canonical source version must be selected.")),
    };
    if let Some(v) = version {
        if v.sha256 != source_hash || v.size_bytes.is_some_and(|n| n != bytes.len() as u64) {
            return Err(invalid(
                "Canonical source does not match the approved version hash and size.",
            ));
        }
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| invalid("Canonical source must be valid UTF-8."))?
        .trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .flexible(version.is_some())
        .from_reader(text.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| invalid("Cannot read CSV header."))?;
    if headers
        .iter()
        .map(|h| {
            if version.is_some_and(|v| v.trim_headers) {
                h.trim()
            } else {
                h
            }
        })
        .collect::<Vec<_>>()
        != HEADERS
    {
        return Err(invalid(
            "CSV columns do not match the canonical seven-column schema.",
        ));
    }
    let mut rows = Vec::new();
    let mut quoting_issue_rows = Vec::new();
    let mut record = csv::StringRecord::new();
    let mut position = reader.position().byte() as usize;
    while reader
        .read_record(&mut record)
        .map_err(|e| invalid(&format!("Structural CSV error: {e}")))?
    {
        let end = reader.position().byte() as usize;
        if quoting_issue(&text.as_bytes()[position..end]) {
            quoting_issue_rows.push(rows.len() + 1);
        }
        position = end;
        rows.push(record.iter().map(str::to_owned).collect());
    }
    if rows.is_empty() {
        return Err(invalid("Canonical source is empty."));
    }
    if version.is_some_and(|v| v.data_rows != rows.len()) {
        return Err(invalid(
            "Canonical source row count does not match the approved version.",
        ));
    }
    Ok((
        Profile {
            import_policy: config.import_policy,
            quoting_issue_rows,
            source_version: version.map(|v| v.version.clone()),
            apply_allowed: version.is_none_or(|v| v.apply_allowed),
            malformed_rows: rows
                .iter()
                .enumerate()
                .filter_map(|(i, r): (usize, &Vec<String>)| {
                    (r.len() != HEADERS.len()).then_some(i + 1)
                })
                .collect(),
            path: path.to_string_lossy().into_owned(),
            hash: source_hash,
            size: bytes.len() as u64,
            total_rows: rows.len(),
            columns: HEADERS.iter().map(|s| s.to_string()).collect(),
            profiled_at: now(),
        },
        rows,
    ))
}
