use serde::Deserialize;

const GITHUB_API_URL: &str = "https://api.github.com/repos/RustedBytes/hrs-launcher/releases/latest";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable { latest_version: String, url: String },
    CheckFailed(String),
}

/// Check if a new version is available on GitHub releases.
/// 
/// # Errors
/// Returns error string if the GitHub API request fails or the response is invalid.
pub async fn check_for_updates(current_version: &str) -> Result<UpdateStatus, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(GITHUB_API_URL)
        .header("User-Agent", "hrs-launcher")
        .send()
        .await
        .map_err(|err| format!("Failed to check for updates: {err}"))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status: {}",
            response.status()
        ));
    }
    
    let release: ReleaseInfo = response
        .json()
        .await
        .map_err(|err| format!("Failed to parse release info: {err}"))?;
    
    let latest_version = normalize_version(&release.tag_name);
    let current = normalize_version(current_version);
    
    if compare_versions(&latest_version, &current) == VersionComparison::Greater {
        Ok(UpdateStatus::UpdateAvailable {
            latest_version: release.tag_name.clone(),
            url: release.html_url.clone(),
        })
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum VersionComparison {
    Greater,
    Equal,
    Less,
}

/// Normalize version string by removing 'v' prefix and cleaning up.
fn normalize_version(version: &str) -> String {
    version.trim().trim_start_matches('v').to_owned()
}

/// Compare two semantic versions.
/// Returns Greater if `a` > `b`, Equal if `a` == `b`, Less if `a` < `b`.
fn compare_versions(a: &str, b: &str) -> VersionComparison {
    let parts_a: Vec<u32> = parse_version_parts(a);
    let parts_b: Vec<u32> = parse_version_parts(b);
    
    let max_len = parts_a.len().max(parts_b.len());
    
    for i in 0..max_len {
        let a_part = parts_a.get(i).copied().unwrap_or(0);
        let b_part = parts_b.get(i).copied().unwrap_or(0);
        
        if a_part > b_part {
            return VersionComparison::Greater;
        } else if a_part < b_part {
            return VersionComparison::Less;
        }
    }
    
    VersionComparison::Equal
}

/// Parse version string into parts (e.g., "0.1.5" -> [0, 1, 5]).
fn parse_version_parts(version: &str) -> Vec<u32> {
    version
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn normalizes_version_strings() {
        assert_eq!(normalize_version("v0.1.5"), "0.1.5");
        assert_eq!(normalize_version("0.1.5"), "0.1.5");
        assert_eq!(normalize_version("  v1.2.3  "), "1.2.3");
    }
    
    #[test]
    fn parses_version_parts_correctly() {
        assert_eq!(parse_version_parts("0.1.5"), vec![0, 1, 5]);
        assert_eq!(parse_version_parts("1.2.3"), vec![1, 2, 3]);
        assert_eq!(parse_version_parts("10.0"), vec![10, 0]);
        assert_eq!(parse_version_parts("invalid"), Vec::<u32>::new());
    }
    
    #[test]
    fn compares_versions_correctly() {
        assert_eq!(
            compare_versions("0.1.6", "0.1.5"),
            VersionComparison::Greater
        );
        assert_eq!(
            compare_versions("0.2.0", "0.1.5"),
            VersionComparison::Greater
        );
        assert_eq!(
            compare_versions("1.0.0", "0.9.9"),
            VersionComparison::Greater
        );
        assert_eq!(
            compare_versions("0.1.5", "0.1.5"),
            VersionComparison::Equal
        );
        assert_eq!(
            compare_versions("0.1.4", "0.1.5"),
            VersionComparison::Less
        );
        assert_eq!(
            compare_versions("0.1", "0.1.0"),
            VersionComparison::Equal
        );
    }
}
