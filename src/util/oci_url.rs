//! Utility for constructing OCI chart references.

/// Builds a fully-qualified OCI chart reference from a repository URL and chart name.
///
/// Handles:
/// - Ensuring `oci://` prefix is present
/// - Avoiding chart name duplication when the URL already ends with the chart name
/// - Trimming trailing slashes
///
/// # Examples
/// ```
/// use vesshelm::util::oci_url::build_oci_chart_ref;
///
/// assert_eq!(
///     build_oci_chart_ref("oci://ghcr.io/hyperspike/valkey-operator", "valkey-operator"),
///     "oci://ghcr.io/hyperspike/valkey-operator"
/// );
/// assert_eq!(
///     build_oci_chart_ref("oci://ghcr.io/org", "my-chart"),
///     "oci://ghcr.io/org/my-chart"
/// );
/// ```
pub fn build_oci_chart_ref(repo_url: &str, chart_name: &str) -> String {
    let mut url = if repo_url.starts_with("oci://") {
        repo_url.to_string()
    } else {
        format!("oci://{}", repo_url)
    };

    let chart_suffix = format!("/{}", chart_name);
    if !url.ends_with(&chart_suffix) {
        url = format!("{}/{}", url.trim_end_matches('/'), chart_name);
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_already_has_chart_name() {
        assert_eq!(
            build_oci_chart_ref(
                "oci://ghcr.io/hyperspike/valkey-operator",
                "valkey-operator"
            ),
            "oci://ghcr.io/hyperspike/valkey-operator"
        );
    }

    #[test]
    fn test_url_needs_chart_name_appended() {
        assert_eq!(
            build_oci_chart_ref("oci://ghcr.io/org", "my-chart"),
            "oci://ghcr.io/org/my-chart"
        );
    }

    #[test]
    fn test_url_without_oci_prefix() {
        assert_eq!(
            build_oci_chart_ref("ghcr.io/org", "my-chart"),
            "oci://ghcr.io/org/my-chart"
        );
    }

    #[test]
    fn test_url_with_trailing_slash() {
        assert_eq!(
            build_oci_chart_ref("oci://ghcr.io/org/", "my-chart"),
            "oci://ghcr.io/org/my-chart"
        );
    }

    #[test]
    fn test_partial_suffix_not_matching() {
        // "my-valkey-operator" ends with "valkey-operator" substring
        // but NOT as a path segment. The full suffix is "/valkey-operator".
        assert_eq!(
            build_oci_chart_ref("oci://ghcr.io/my-valkey-operator", "valkey-operator"),
            "oci://ghcr.io/my-valkey-operator/valkey-operator"
        );
    }
}
