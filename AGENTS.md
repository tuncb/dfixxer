After rust code changes:
  - build the app then run cargo test, cargo fmt, cargo clippy

After we change the behavior of the application:
  - Add unit tests
  - If the behavior change can be checked using file output add a test in one or more of the end2end tests under test-data folder.
  - If the behavior change does not change file output but just the output to stdout of the app, run the app to examine the command output to make sure the changes are correct.

After we change command line parameters update the readme.md