<p align="center">
<img src="logo.png" alt="Vesshelm logo" width="450"/>
</p>

Vesshelm is a modern, reliable, and user-friendly tool for managing Helm charts in a GitOps way.

Designed as an alternative to  [helmfile](https://github.com/helmfile/helmfile), and [helm-freeze](https://github.com/Qovery/helm-freeze). It simplifies the process of syncing charts from various sources (Helm repositories, Git repositories, OCI registries, and local files) and deploying them with confidence. Everything is local-first and reproducible with a great user experience.

## Features

- **Multi-Source Support**: Download charts from Helm repositories, Git repositories, OCI registries, or use local folders.
- **Local Sync**: Downloads charts locally first, ensuring you have a frozen, reproducible state.
- **Dependency Management**: Supports chart dependencies and deploys them in the correct topological order.
- **Smart Deployments**:
  - **Diff-Aware**: Automatically runs `helm diff` (if enabled) before deploying.
  - **Values Management**: Supports `values_files` and inline `values` override.
  - **Templating**: Support Jinja2 templating in values files with global variables.
  - **Filtering**: Deploy specific charts with `--only`.
- **Life-Cycle Management**:
  - **Check Updates**: Easily identify and apply newer chart versions.
  - **Uninstall**: Clean up releases with a single command.
- **Ease of Use**:
  - **Interactive**: `add` command to easily onboard new charts from ArtifactHub, Git, or OCI.
  - **Autocompletion**: Comprehensive shell completion for commands and chart names.
- **Secrets Management**: Integrated SOPS support for encrypted values files.
- **User Interface**: Beautiful interactive CLI with progress bars for syncing and deployment.
- **Configuration**: Strongly typed `vesshelm.yaml` configuration with validation.

## Comparison with Other Tools

Compared to Helm-freeze:
* Manage chart deployments
* Graph dependencies
* Check updates
* Uninstall

Compared to helmfile:
* Better user experience
* Designed for simpler usage (KISS)
* Check updates

## Installation

### Via Homebrew (macOS/Linux)

```bash
brew tap deimosfr/vesshelm
brew install vesshelm
```

### Binary Release

Download the latest binary for your platform from the [Releases page](https://github.com/deimosfr/vesshelm/releases).

> **Note for macOS users**: You will encounter a "System Security" error when running the binary, you may need to clear the quarantine attribute:
> ```bash
> xattr -d com.apple.quarantine /opt/homebrew/bin/vesshelm
> ```


### From Source

Ensure you have Rust installed:

```bash
cargo install --path .
```

## Usage

### 1. Initialize

Sets up Vesshelm in the current directory:
- Checks if `helm` is installed and available in your PATH.
- Checks if `helm-diff` is installed and available in your PATH otherwise it will install it.
- Creates a `vesshelm.yaml` configuration file with sensible defaults if one doesn't exist.

```bash
vesshelm init
```

### 2. Add Chart

Interactively add a new chart to your configuration:
- Supports Artifact Hub, Git repositories, and OCI registries.
- Automatically detects repository URL and chart version.
- Checks for duplicate charts.
- Updates `vesshelm.yaml` and offers to sync immediately.

```bash
vesshelm add
```

### 3. Sync Charts

Downloads charts to your local destinations to ensure reproducible deployments:
- Fetches charts from Helm repositories, Git repositories, or OCI registries.
- Updates local Helm repository caches (`helm repo update`).
- Downloads chart dependencies (`helm dependency build`).
- Stores charts locally in the configured `destinations`.

```bash
$ vesshelm sync
==> Syncing 17 charts...
 WARN: Failed to update helm repos: Failed to update helm repos: Error: no repositories found. You must add one before updating
 [OK]   cilium (Helm)
 [SKIP] cilium-config (local chart)
 [SKIP] custom-priority-classes (local chart)
 [OK]   openebs (Helm)
 [OK]   cert-manager (Helm)
 [SKIP] cert-manager-config (local chart)
 [FAIL] csi-driver-smb: Failed to clone git repo https://github.com/kubernetes-csi/csi-driver-smb.git
 [SKIP] csi-driver-smb-config (local chart)
 [OK]   mariadb-operator (Helm)
 [OK]   mariadb-operator-crds (Helm)
 [SKIP] mariadb-config (local chart)
 [SKIP] ddns-updater (local chart)
 [OK]   dnsmasq-k8s (Helm)
 [SKIP] gateways (local chart)
 [OK]   metrics-server (Helm)
 [OK]   adguard-home (Helm)
 [OK]   external-dns (Helm)

  [00:00:06] [████████████████████████████████████████] 17/17 (100%) Sync completed
Summary:
  Synced:  9
  Failed:  1
  Skipped: 9
Error: Some charts failed to sync
```

Or specific charts:

```bash
vesshelm sync --only my-chart
```

### 4. Deploy

Orchestrates the deployment of your charts to Kubernetes:
- Resolves chart dependencies to determine the correct deployment order.
- Runs `helm diff` (if enabled) to preview changes before applying.
- Standardizes Helm arguments using your configuration.
- Supports values files and inline value overrides.

```bash
# Deploy all charts
vesshelm deploy

# Perform a dry-run (diff only)
vesshelm deploy --dry-run

# Deploy specific charts
vesshelm deploy --only my-charts
```

Output example:

```bash
$ vesshelm deploy --dry-run
🚀  Starting deployment...
📦  Deploying chart mariadb-operator-crds
🔎  helm diff upgrade --suppress-secrets --allow-unreleased mariadb-operator-crds ./charts/mariadb-operator-crds -n mariadb-operator
⏭  No changes for mariadb-operator-crds. Skipping.
📦  Deploying chart custom-priority-classes
🔎  helm diff upgrade --suppress-secrets --allow-unreleased custom-priority-classes charts/custom-priority-classes -n kube-system
⏭  No changes for custom-priority-classes. Skipping.

  [00:00:08] [████████████████████████████████████████] 3/3 (100%) Deployment ended

Summary:
  Deployed: 0
  Failed:   0
  Skipped:  3
  Ignored:  0
```

### 5. Visualize

Generates a visual representation of your chart dependencies:
- Displays a dependency tree showing how charts relate to each other.

```bash
$ vesshelm graph
==> Calculating dependency graph...
Deployment Graph
├─ cilium
│  └─ cilium-config
│     ├─ adguard-home
│     ├─ cert-manager
│     │  └─ cert-manager-config
│     │     └─ gateways
│     ├─ ddns-updater
│     ├─ dnsmasq-k8s
│     ├─ external-dns
│     ├─ mariadb-operator
│     │  └─ mariadb-config
│     ├─ metrics-server
│     └─ openebs
│        ├─ csi-driver-smb
│        │  └─ csi-driver-smb-config
│        └─ mariadb-operator
│           └─ mariadb-config
├─ custom-priority-classes
└─ mariadb-operator-crds
   └─ mariadb-operator
      └─ mariadb-config
```

### 6. Check Updates

Scans your repositories (Helm/Git) for newer chart versions:
- Compares your local version against the upstream repository.
- Support semantic versioning comparison.
- Can automatically apply version updates to your `vesshelm.yaml`.

```bash
# Check all charts
vesshelm check-updates

# Check specific charts
vesshelm check-updates --only my-chart

# Update new versions to vesshelm.yaml
vesshelm check-updates --apply

# Update and sync immediately
vesshelm check-updates --apply-sync
```

Output example:

```bash
$ vesshelm check-updates
🔄 Updating Helm repositories...
Hang tight while we grab the latest from your chart repositories...
...Successfully got an update from the "metrics-server" chart repository
...Successfully got an update from the "external-dns" chart repository
...
Update Complete. ⎈Happy Helming!⎈

🔍 Checking for updates...
checking cilium... Up to date
checking cilium-config... Skipped (local/git/oci)
checking custom-priority-classes... Skipped (local/git/oci)
checking openebs... Up to date
checking cert-manager... Up to date
checking cert-manager-config... Skipped (local/git/oci)
checking csi-driver-smb... Skipped (local/git/oci)
checking csi-driver-smb-config... Skipped (local/git/oci)
checking mariadb-operator... Up to date
checking mariadb-operator-crds... Up to date
checking mariadb-config... Skipped (local/git/oci)
checking ddns-updater... Skipped (local/git/oci)
checking dnsmasq-k8s... Up to date
checking gateways... Skipped (local/git/oci)
checking metrics-server... Up to date
checking adguard-home... Up to date
checking external-dns... Up to date

Run with --apply to apply changes.
```

### 7. Uninstall

Safely removes chart releases from your cluster:
- Checks if other charts depend on the one you are uninstalling.
- Warns you about dependencies before proceeding.
- Runs `helm uninstall` for the specified chart.

```bash
# Uninstall a specific chart
vesshelm uninstall my-chart

# Interactive selection if name is omitted
vesshelm uninstall

# Skip confirmation
vesshelm uninstall my-chart --no-interactive
```

### 8. Delete

Removes a chart from your local configuration (`vesshelm.yaml`) and filesystem.
- Offers interactive chart selection if multiple charts share the same name (e.g. different namespaces) or if no name is provided.
- Can optionally uninstall the Helm release from the cluster during the process.
- Updates dependencies in `vesshelm.lock`.
- Non-destructively updates `vesshelm.yaml` preserving comments and formatting.

```bash
# Delete a specific chart
vesshelm delete my-chart

# Interactive selection
vesshelm delete
```

### 9. Validate

Ensures your configuration is correct before running operations:
- validates YAML syntax and structure.
- Checks for duplicate chart names.
- Verifies that referenced repositories and destinations exist.
- Checks that values files exist.

```bash
$ vesshelm validate
Configuration is valid
```

### 10. Autocompletion

Generate shell completion scripts for your shell (bash, zsh, fish, etc.).

```bash
# For Zsh
source <(vesshelm completion zsh)

# For Fish
vesshelm completion fish | source

# For Bash
source <(vesshelm completion bash)
```

## Configuration (`vesshelm.yaml`)

Vesshelm is configured via a YAML file. Here is a comprehensive example:

```yaml
# Define where to fetch charts from
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
  - name: my-git-repo
    url: https://github.com/my-org/my-charts.git
    type: git
  - name: my-oci-registry
    url: oci://registry.example.com/charts
    type: oci

# Define where to download charts to locally
destinations:
  - name: default
    path: ./charts
  - name: custom
    path: ./custom_charts

# Define global variable files for Jinja2 interpolation in values files
variable_files:
  - vars/common.yaml
  - vars/region-{{ env.REGION }}.yaml

# Define secrets files for SOPS integration
# Encrypted files will be automatically decrypted before use
secrets_files:
  - secrets.yaml
  - secrets/{{ env.ENV }}.yaml

# Global Helm settings
vesshelm:
  # Base arguments for helm commands
  helm_args: "upgrade --install {{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --create-namespace"
  # Enable diff before deploy
  diff_enabled: true
  # Optional custom diff command
  diff_args: "diff upgrade --suppress-secrets --allow-unreleased {{ name }} {{ destination }} -n {{ namespace }}"
  # Pause deployment on error for debugging (interactive mode only). Defaults to true.
  deploy_debug_pause: true

charts:
  # 1. Standard Helm Repo Chart
  - name: nginx-ingress
    repo_name: stable
    version: 1.41.3
    namespace: ingress
    dest: default
    values_files:
      - overrides/nginx.yaml
    values:
      - controller.replicaCount: 2
    # Append additional arguments to the helm command
    helm_args_append: "--timeout 600s"

  # 2. Chart from Git Repository
  - name: my-internal-service
    repo_name: my-git-repo
    version: main                 # Branch, tag, or commit
    chart_path: services/backend  # Path inside the git repo
    namespace: backend
    # Override the default destination
    dest: custom

  # 3. Local Chart (No repo_name/version needed)
  - name: local-debug-chart
    chart_path: ./local-charts/debug-tool
    namespace: debug
    no_deploy: true               # Do not deploy this chart (useful for reference/base charts)

  # 4. Chart with Dependencies
  - name: my-app
    repo_name: stable
    version: 0.1.0
    no_sync: true                 # Skip syncing if made a temporary manual change for example
    namespace: apps
    depends:
      - my-internal-service       # Will ensure 'my-internal-service' is deployed first
```

### Vesshelm configuration variable interpolation

In `vesshelm.helm_args` and `vesshelm.diff_args`, you can use the following variables:
- `{{ name }}`: Chart name
- `{{ destination }}`: Destination path (e.g., `./charts`)
- `{{ namespace }}`: Namespace
- `{{ version }}`: Chart version (empty for local charts)
- `{{ chart_path }}`: Full path to the chart (handles both remote downloads and local paths)

### Values Files Interpolation

You can use Jinja2 templating in your `values_files`.
Variables are loaded from files defined in `variable_files` in `vesshelm.yaml`.

Example `vesshelm.yaml`:
```yaml
variable_files:
  - common.yaml
```

Example `common.yaml`:
```yaml
domain: example.com
region: us-east-1
```

Example `values.yaml`:
```yaml
ingress:
  host: api.{{ domain }}
  annotations:
    region: {{ region }}
```

### Configuration Reference

#### Variables Options

| Field | Type | Description |
|-------|------|-------------|
| `variable_files` | list | Optional list of paths to YAML files containing global variables for Jinja2 interpolation. |

#### Repository Options

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | **Required**. A unique name for the repository. Used to reference it in charts using `repo_name`. |
| `url` | string | **Required**. The URL of the repository (HTTP/S, Git URL, or OCI registry). |
| `type` | string | The type of repository. One of: `helm` (default), `git`, `oci`. |

#### Destination Options

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | **Required**. A unique name for the destination. Used to reference it in charts using `dest`. |
| `path` | string | **Required**. The local filesystem path where charts will be downloaded. |

#### Secrets Options

| Field | Type | Description |
|-------|------|-------------|
| `secrets_files` | list | Optional list of paths to SOPS-encrypted YAML files. |

#### Global Helm Options (`vesshelm`)

| Field | Type | Description |
|-------|------|-------------|
| `helm_args` | string | **Required**. Base arguments template for Helm commands (e.g., `upgrade --install ...`). |
| `diff_enabled` | bool | Whether to run `helm diff` before deploying. Defaults to `true`. |
| `diff_args` | string | Optional custom arguments template for the `helm diff` command. |
| `deploy_debug_pause` | bool | Whether to pause on deployment error for debugging (interactive mode only). Defaults to `true`. |

#### Chart Options

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | **Required**. The name of the chart (release name). |
| `namespace` | string | **Required**. The Kubernetes namespace to deploy to. |
| `repo_name` | string | Name of the repository to fetch from (must match a repository in `repositories`). |
| `version` | string | Version of the chart to fetch (or branch/tag for Git). |
| `chart_path` | string | Path to the chart. For Git repos, it's the relative path inside the repo. For local charts, it's the local path. |
| `dest` | string | The destination name for downloading the chart (must match a destination in `destinations`). Defaults to the first defined destination. |
| `values_files` | list | List of paths to Helm values files. |
| `values` | list | Inline values override (list of maps, e.g. `key: value`). |
| `helm_args_append` | string | Append additional arguments to the Helm command for this specific chart. |
| `helm_args_override` | string | Completely parameters of the Helm command for this specific chart (ignores global `vesshelm.helm_args`). |
| `no_sync` | bool | If `true`, skips the sync/download step for this chart. |
| `no_deploy` | bool | If `true`, skips the deploy step for this chart. |
| `no_interpolation` | bool | If `true`, disables Jinja2 interpolation for this chart's values files. |
| `depends` | list | List of chart names that this chart depends on. Controls deployment order. |

## Contribution

Contributions are welcome! Please open an issue or submit a pull request.

`just` is used to manage development tasks:
- Run tests and checks: `just test` (runs `pre-commit`)
- Fix linting issues: `just fix`
- Install locally: `just install`
