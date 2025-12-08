# Co-locate Tests with Code implementation

## Goal
Move all tests from the `tests/` directory to be co-located with the code they test within the `src/` directory. This ensures better locality, allows testing of private modules if needed, and aligns with the user's specific preference for file organization.

## Terms
- **Co-located Tests**: Tests residing in the same source file as the implementation (typically in a `#[cfg(test)] mod tests` module).

## Status
- **Discussion**: Open
- **Implementation**: Pending Approval
