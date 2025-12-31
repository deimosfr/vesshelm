# Design: Vesshelm Add Command

## Overview
The `vesshelm add` command provides an interactive way to add charts to `vesshelm.yaml`. It leverages Artifact Hub to discover chart metadata.

## User Interaction Flow
1.  **Start**: User runs `vesshelm add`.
2.  **Input URL**: Prompt "Enter Artifact Hub URL: ". User enters URL (e.g., `https://artifacthub.io/packages/helm/dnsmasq-k8s/dnsmasq-k8s`).
3.  **Discovery**: System parses URL or queries Artifact Hub API to find:
    *   Repository Name (e.g., `dnsmasq-k8s`)
    *   Repository URL (e.g., `https://deimosfr.github.io/dnsmasq-k8s/`)
    *   Chart Name (e.g., `dnsmasq-k8s`)
    *   Latest Version (e.g., `1.4.1`)
4.  **Repository Check**:
    *   If Repo URL exists in `repositories`: Use existing repo name.
    *   If not: Add to candidate `repositories` list. Default name from Artifact Hub, but ensure uniqueness.
5.  **Chart Configuration**:
    *   **Name**: Default to Chart Name. Prompt "Chart Name [default]: ". Ensure uniqueness in `charts`.
    *   **Namespace**: Prompt "Namespace: ".
6.  **Preview**: Show the proposed YAML additions (Repository and Chart).
7.  **Confirm**: Prompt "Add to config? [y/N]".
8.  **Action**:
    *   If Yes: Write to `vesshelm.yaml`.
    *   Run validation.
    *   Prompt "Sync now? [y/N]".
    *   If Sync Yes: Run `vesshelm sync`.
    *   If No: Exit.

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
