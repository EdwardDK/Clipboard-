/// Best-effort local classification. It intentionally avoids logging the input.
pub fn is_likely_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let markers = ["password", "passwd", "api_key", "access_token", "refresh_token", "private key", "connection string", "mongodb://", "postgres://"];
    markers.iter().any(|marker| lower.contains(marker))
        || value.starts_with("-----BEGIN ") && value.contains("PRIVATE KEY-----")
        || high_entropy_token(value)
}

fn high_entropy_token(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() >= 32 && trimmed.len() <= 512 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || "-_.+/=".contains(c)) && trimmed.chars().any(|c| c.is_ascii_lowercase()) && trimmed.chars().any(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}
