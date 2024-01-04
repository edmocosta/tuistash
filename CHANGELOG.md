## 0.1.3
- Added a new `flows` view built on top of the latest Logstash flow metrics.
- **BREAKING**: Renamed the `view` command to `tui`.
- Changed to execute the `tui` command by default when no specific command is supplied.
- Migrated from `tui` to `ratatui` and bumped a few dependencies.
- Changed `pipelines` charts to continuously aggregate data, even if the chart isn't being displayed.
- Added `worker millis per event` chart on the pipeline's plugin details.
- Reorganized TUI shortcuts and other design changes.
- Added license file