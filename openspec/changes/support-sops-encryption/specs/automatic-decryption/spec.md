# Spec: Automatic SOPS Decryption

## ADDED Requirements

### Requirement: Automatic detection and transparent decryption of SOPS files
The system MUST transparently detect if a file is encrypted with SOPS and decrypt it using the `sops` binary at runtime before processing its content as YAML or templating it.
#### Scenario: Verify detection of SOPS encrypted files
Given a file that contains a root `sops` key
When `vesshelm` attempts to read this file
Then it should identify it as an encrypted file

#### Scenario: Verify failure when sops binary is missing
Given a SOPS encrypted file is referenced in variables or overrides
And the `sops` binary is NOT in the system PATH
When `vesshelm` runs a deployment
Then it should fail with a descriptive error message indicating `sops` is missing

#### Scenario: Verify transparent decryption
Given a valid SOPS encrypted file with accessible keys
When `vesshelm` loads this file as a variable file or value override
Then it should decrypt the content in-memory and use the plain text values

#### Scenario: Verify template rendering on decrypted content
Given a SOPS encrypted file that contains MiniJinja templates (e.g. `{{ var }}`) inside the encrypted data
When `vesshelm` renders this file
Then it should decrypt first, and THEN process the templates
