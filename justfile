default:
  just --list

# Run all pre-commit hooks
test:
    pre-commit run --all-files

# Build and install locally for testing purposes (override brew install destination file)
install:
    cargo build ; sudo cp target/debug/vesshelm /opt/homebrew/bin/vesshelm

# Fix code style and clippy warnings (dirty enabled)
fix:
    cargo fmt
    cargo clippy --fix --allow-staged --allow-dirty

# Show last openspec remaining actions
spec:
    watch 'openspec view | head -20'
