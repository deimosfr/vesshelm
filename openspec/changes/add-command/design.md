# Design: Vesshelm Add Command

## Overview
The `vesshelm add` command provides an interactive way to add charts to `vesshelm.yaml`. It leverages Artifact Hub to discover chart metadata.

## User Interaction Flow
1.  **Start**: User runs `vesshelm add`.
2.  **Source Type Selection**: Prompt "Select source type: [Artifact Hub (default), Git, OCI]".
3.  **Source Specific Flow**:

    ### A. Artifact Hub (Default)
    1.  **Input URL**: Prompt "Enter Artifact Hub URL: ".
    2.  **Discovery**: Parse URL/API to find Repo Name/URL, Chart Name, Version.

    ### B. Git
    1.  **Input Repo URL**: Prompt "Enter Git Repository URL: " (e.g., `https://github.com/kubernetes-csi/csi-driver-smb`).
    2.  **Input Chart Path**: Prompt "Enter path to chart in repo: " (e.g., `charts/v1.19.1/csi-driver-smb`).
    3.  **Input Version**: Prompt "Enter Version (commit/tag/branch): " (e.g., `v1.0.0` or SHA).
    4.  **Derive Details**:
        *   Chart Name: Default to last segment of Chart Path or Repo Name.
        *   Repo Name: Default to last segment of Repo URL.

    ### C. OCI
    1.  **Input OCI URL**: Prompt "Enter OCI URL: " (e.g., `oci://code.forgejo.org/forgejo-helm/forgejo`).
    2.  **Input Version**: Prompt "Enter Version: " (e.g., `15.0.3`).
    3.  **Derive Details**:
        *   Chart Name: Last segment of OCI URL.
        *   Repo URL: Use the OCI URL base or full URL depending on structure.
            *   *Note*: For `helm install my-forgejo oci://code.forgejo.org/forgejo-helm/forgejo`, the repo URL corresponds to the chart URL itself in OCI usually, or the registry. Vesshelm `repositories` entry for OCI typically needs `url` to be the registry/path prefix?
            *   Wait, the user example shows `url: oci://code.forgejo.org/forgejo-helm`. This is the parent of `forgejo`.
            *   SO: If OCI URL is `oci://reg/path/chart`, Repo URL is `oci://reg/path`? Or user inputs it?
            *   Let's check `Helm` behavior: `helm install <name> <chart-ref>`.
            *   For Vesshelm config:
                ```yaml
                repositories:
                  - name: forgejo
                    type: oci
                    url: oci://code.forgejo.org/forgejo-helm
                charts:
                  - name: forgejo
                    repo_name: forgejo
                ```
            *   So we need to prompt or deduce the "Repo URL" from the "Chart URL".
            *   **Logic**: If user gives `oci://code.forgejo.org/forgejo-helm/forgejo`, we can try to deduce repo as `oci://code.forgejo.org/forgejo-helm`. OR prompt?
            *   User req: "2.1 Ask for oci url". Example: `oci://code.forgejo.org/forgejo-helm/forgejo`.
            *   The resulting repo config url: `oci://code.forgejo.org/forgejo-helm`.
            *   So we strip the last segment?
            *   Let's verify logic: `helm install my-forgejo oci://code.forgejo.org/forgejo-helm/forgejo`.
            *   Here `oci://code.forgejo.org/forgejo-helm/forgejo` is the chart reference.
            *   If using a repo alias, we need `helm repo add my-repo oci://code.forgejo.org/forgejo-helm`. Then `helm install my-chart my-repo/forgejo`.
            *   So yes, splitting the URL is a safe initial guess.

4.  **Repository Check**:
    *   Check for existing repo by URL (Git/OCI/Helm).
    *   If new: Prompt for Repo Name (default derived from URL).
5.  **Chart Configuration**:
    *   **Name**: Default to derived Chart Name.
    *   **Namespace**: Prompt "Namespace: ".
6.  **Preview**: Show proposed changes.
7.  **Confirm**: Prompt "Add to config?".
8.  **Action**: Update `vesshelm.yaml` and validate.

## Artifact Hub Integration
We will likely need to scrape the Artifact Hub page or use their API if available and simple. The user mentioned an API `https://artifacthub.io/docs/api/#/`.
Public API: `GET /api/v1/packages/helm/{repo}/{name}`.
We might need to parse the user provided URL to extract `repo` and `name` to query the API.
User URL format: `https://artifacthub.io/packages/helm/{repo_name}/{package_name}`.

## Configuration Updates
## Configuration Updates
*   **Strategy**: "Smart Append/Insert". Do NOT rewrite the entire file using `serde`. This destroys comments and reorders fields.
*   **Implementation**:
    *   Read `vesshelm.yaml` as text.
    *   Locate the `repositories:` section. Append the new repository entry if needed.
    *   Locate the `charts:` section. Append the new chart entry.
    *   **Formatting**: Ensure the appended YAML respects indentation.
*   **Conciseness**: The appended YAML MUST NOT include fields with `null` values or default values (e.g., `dest`, `chart_path`, `values_files`, `depends`). Only include fields that are explicitly set (Name, Namespace, Repo, Version, Comment).

## Data Structures
New structs might be needed for Artifact Hub API responses.

## Code Structure Refactoring
To improve maintainability, the monolithic `add.rs` will be refactored:

### 1. Source Module (`src/cli/commands/add/source.rs`)
*   **Trait**: `ChartSource`
    *   `fn prompt_details(&self) -> Result<ChartDetails>`
*   **Implementations**:
    *   `ArtifactHubSource`
    *   `GitSource`
    *   `OciSource`
*   **Factory**: `get_source(type: SourceType) -> Box<dyn ChartSource>`

### 2. Config Updater (`src/cli/commands/add/config_updater.rs`)
*   Encapsulate the "Smart Append" logic.
*   `fn add_repository(content: &mut String, repo: &Repository)`
*   `fn add_chart(content: &mut String, chart: &Chart, comments: Option<&str>)`

### 3. Main Command (`src/cli/commands/add/mod.rs`)
*   Orchestrates the flow:
    1.  Prompt Source Selection.
    2.  Get Source Implementation.
    3.  `source.prompt_details()`.
    4.  Prompt Config Details (Namespace, etc.).
    5.  Preview.
    6.  `config_updater::update_file()`.
