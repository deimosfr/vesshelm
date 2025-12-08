## 1. Git Utility Enhancements

- [x] 1.1 Implement `GitUtil::read_file_from_head(path: &Path)` to retrieve file contents from last commit
- [x] 1.2 Implement `GitUtil::get_diff_from_head()` to capture all changes (staged and unstaged) compared to HEAD
- [x] 1.3 Update `GitUtil::is_dirty()` to properly identify if current work is ahead of HEAD

## 2. CLI Command Updates

- [x] 2.1 Add `re_analyze: bool` to `AiUpdateArgs` with shorthand `-R`
- [x] 2.2 Update `ai_update::run` to handle the new `re_analyze` flag
- [x] 2.3 Implement interactive dirty-check confirmation prompt using `interaction.confirm`

## 3. Re-analyze Logic

- [x] 3.1 Implement logical skip of chart sync and version check when `--re-analyze` is active
- [x] 3.2 Implement logic to load "old" configuration from HEAD when re-analyzing
- [x] 3.3 Ensure templates are correctly rendered and diffed against HEAD state in re-analyze mode

## 4. Polish and Validation

- [x] 4.1 Update error messages to suggest `--re-analyze` if appropriate
- [x] 4.2 Verify the workflow by manually running a mock update and subsequent re-analysis
