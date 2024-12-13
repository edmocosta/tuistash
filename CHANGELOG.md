## 0.5.0
 - Added support for vertices and edges files diagnostics.
 - The thread name is now displayed on the traces details block.
 - **BREAKING**: Removed the `default` and `table` output options from the `node` command.
 - Bumped dependencies versions.

## 0.4.0
 - Introduced a new view `Threads`, which relies on the Logstash's hot-threads API to display the busiest threads and their traces.
 - Added the `User-Agent` header to requests so the source can be identified.
 - Minor UI fixes.

## 0.3.0
 - Bumped a few dependencies.
 - Added a command option (`diagnostic-path`) to poll the data from a Logstash diagnostic path.
 - Improved compatibility with older Logstash versions (7.x), which graph API is not supported.
 - The pipeline components view now shows the plugin's pipeline usage and the dropped events percentages.
 - Added a few plugin's extra details on the pipeline view.

## 0.2.0
- Added a new `flows` view built on top of the latest Logstash flow metrics.
- **BREAKING**: Renamed the `view` command to `tui`.
- Changed to execute the `tui` command by default when no specific command is supplied.
- Migrated from `tui` to `ratatui` and bumped a few dependencies.
- Changed `pipelines` charts to continuously aggregate data, even if the chart isn't being displayed.
- Added `worker millis per event` chart on the pipeline's plugin details.
- Reorganized TUI shortcuts and other design changes.
- Added license file