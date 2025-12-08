# Tasks

- [x] Update `ConfigError::ParseFailed` and `ParseFailedWithHint` to include file path
- [x] Update `ConfigError::DefaultDestNotFound` to include chart name
- [x] Update `ConfigError::VersionBoundaryCrossed` to include chart name
- [x] Update `ConfigError::FileWriteFailed` to include file path
- [x] Update all usage sites in `src/config.rs` and `src/util/config_updater.rs` and `src/lock.rs` (if applicable to ConfigError) to provide the new context
- [x] Verify error messages contain the new context
