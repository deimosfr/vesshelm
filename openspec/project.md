# Project Context

## Purpose
It is an alternative to [helm-freeze](https://github.com/Qovery/helm-freeze) and [Helmfile](https://github.com/helmfile/helmfile).
The tool should be able to:
- Download a chart locally from a chart repository, a git repository, or an OCI registry
- Be able to run Helm deployment based on common parameters and local downloaded chart
- It should be able to run in a local environment
- It should be able to run in a CI/CD pipeline
- It should be user friendly with interactive mode CLI on chart fetching with progress bar
- It should be user friendly without interactive mode CLI on chart deployment with progress bar
- It should be able to limit deployment to specific charts
- Maintain a documentation in the README.md file

## Tech Stack
- **Language**: Rust (2021 edition or later)
- **CLI Framework**: [clap](https://crates.io/crates/clap) (Derive API)
- **Async Runtime**: [tokio](https://crates.io/crates/tokio) (for concurrent chart downloads/helm execution)
- **Error Handling**: 
  - [anyhow](https://crates.io/crates/anyhow) for application-level error handling
  - [thiserror](https://crates.io/crates/thiserror) for library/module-level error definitions
- **Serialization**: [serde](https://crates.io/crates/serde) with [serde_yaml](https://crates.io/crates/serde_yaml) for config parsing
- **Progress**: [indicatif](https://crates.io/crates/indicatif) for progress bars
- **Platform**: Cross-platform support (Linux, macOS, Windows)

## Project Conventions

### Code Style
- You're an expert Rust developer, use coding best practices
- Use rustfmt
- Should always compile without warnings
- Should always pass clippy
- Test should never have warnings
- The code should be well documented with tests and rustdoc

### Architecture Patterns
- The code should be professional grade and well structured.
- **Error Handling**: Use `Result` types extensively. Bubble up errors with context using `anyhow::Context`.
- **Async/Await**: Use `async`/`await` for I/O bound tasks (network, disk).
- **Configuration**: Strongly typed configuration structs derived from `serde::Deserialize`.
- **Modularity**: Separation of concerns between CLI interface, configuration parsing, and core logic.

### Testing Strategy
- I want to have unit tests and integration tests.
- I want to have tests that cover all the code.
- I want tests to run inside GitHub actions.
- I want tests to run on pull requests.
- Tests should be in the same files than the code they test.

### Git Workflow
- I want to use gitflow, nothing specific

## Domain Context
- The tool should be able to download a chart locally from a chart repository, a git repository, or an OCI registry
- The tool should be able to run Helm deployment based on common parameters and local downloaded chart
- The tool should be able to run in a local environment
- The tool should be able to run in a CI/CD pipeline
- The tool should be user friendly with interactive mode CLI on chart fetching with progress bar
- The tool should be user friendly without interactive mode CLI on chart deployment with progress bar
- The tool should be able to limit deployment to specific charts
- The tool should be able to validate the configuration file (vesshelm.yaml)
- I want every commands to be clearly separated. The code structure should be clear enough to avoid having to read the code to understand what is happening.

## Important Constraints
- Only write safe code!
- It's ok to use helm command, otherwise, use rust libs and avoid at all cost to use other binaries.

I want to keep something simple and easy to use. I don't want to overcomplicate it. It should be easy to read and understand like helm-freeze:
```
charts:
    # Chart name
  - name: cert-manager
    # Chart version
    version: v1.7.0
    # The repo to use (declared below in the repos section)
    repo_name: jetstack
    # No destinations is declared, the default one will be used
    comment: "You can add comments"
  - name: cert-manager
    # Chart version
    version: v1.8.0
    # The repo to use (declared below in the repos section)
    repo_name: jetstack
    # Override the folder path
    dest_folder_override: cert-manager-1.8
  - name: fluent-bit
    repo_name: lifen
    version: 2.8.0
    # If you temporary want to stop syncing a specific chart
    no_sync: true
  - name: nginx-ingress
    # No repo_name is specified, stable will be used
    version: 1.35.0
    # Change the destination to another one (declared in destinations section)
    dest: custom
  - name: pleco
    repo_name: git-repo
    # When using a git repo, chart_path is mandatory, you need to specify the chart folder path
    chart_path: /charts/pleco
    dest: custom
    # Set git reference
    version: 5e05faddb0fde1f5ddd822c3d3ba72925f094e67

repos:
    # Stable is the default one
  - name: stable
    url: https://charts.helm.sh/stable
  - name: jetstack
    url: https://charts.jetstack.io
  - name: lifen
    url: https://honestica.github.io/lifen-charts
  - name: git-repo
    url: https://github.com/Qovery/pleco.git
    # If you want to directly use a chart folder in a git repo, set type to git
    type: git

destinations:
  - name: default
    path: /my/absolute/path
  - name: custom
    path: ./my/relative/path
```

- Vesshelm should have an init command to check if helm is installed and exit with error if not
- Vesshelm should init a default config file if no one is present
- Vesshelm should have a version command to display the version of the tool
- Vesshelm should have a help command to display the help of the tool
- Vesshelm should have a completion command to display the completion of the tool
- Vesshelm should have a completion install command to install the completion of the tool

In addition to the helm-freeze config, vesshelm should manage helm args and five possibilities to append or override them:
```
helm:
    # helm upgrade --install
    args: "{{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure"
    
charts:
  - comment: https://artifacthub.io/packages/helm/cilium/cilium
    name: cilium
    repo_name: cilium
    version: 1.19.0-pre.3
    namespace: kube-system
    values_files:
        - charts_overrides/cilium.yaml
    helm_args_append: "--create-namespace"
    destination_override: custom
  - name: cert-manager
    namespace: namespace_name
    version: v1.7.0
    repo_name: jetstack
    depends:
        - cilium
    values_files:
        - charts_overrides/cert-manager.yaml
    values:
        - key: value
    comment: "You can add comments"
    helm_args_override: "{{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure --create-namespace"

repositories:
    # Stable is the default one
  - name: stable
    url: https://charts.helm.sh/stable
  - name: cilium
    url: https://helm.cilium.io
  - name: jetstack
    url: https://charts.jetstack.io

destination:
  - name: default
    path:./charts 
```

As you can see in the exemple, it should be able to interpolate values in the helm args based on the chart configuration.

## External Dependencies
- Helm

