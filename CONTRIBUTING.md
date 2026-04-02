# Contributing

When contributing to this repository, you'll have more luck with getting PRs approved if you come chat with us in the
Discord server and letting us know about what you are fixing/adding.
Keep in mind that clippy, rustfmt and cargo-audit are enforced on CI, so make sure your code passes these checks.

## Pull Request Process

1. Make sure all tests and lints pass. PRs that don't pass CI will be rejected if your code is the cause of the failing
   tests/lints.
2. Make sure all necessary files are also included and not using absolute paths.
3. Include a sufficient explanation of your PR. What is it adding/fixing, why does this feature need to be added/fixed,
   who have you discussed this with, etc. If these questions were answered in a conversation on this Discord, mention
   who you talked with and what consensus was reached. Unexplained PRs will rarely be accepted.
4. Check again that tests pass.
5. Check that Clippy passes with no issues. `cargo clippy --all-targets -- -Dwarnings` is used on CI.
6. Check that Rustfmt passes with no issues. `cargo fmt --all -- --check` is used on CI.
7. Check that Cargo-audit passes with no issues. `cargo audit` is used on CI.
8. Submit PR.

> [!NOTE]
> There is a template you will be prompted to fill out when you create a PR. This is not a hard requirement, but is a
> good starting point for making sure you include all the necessary information in your PR.

### Code rules

1. Tests that only generate/dump data must be `#[ignore]`d. These tests are not useful for CI and should not be run.
2. No absolute paths. This will break the CI and make it harder to run the code on different machines.
3. Try not to use `unwrap()`, do proper error handling. If at all possible, try to use `expect()` in place of `unwrap()`
   so that if the code does panic, it will be easier to find the source of the panic. If you are sure that the code will
   never panic, you can use `unwrap()`, but it is generally better to use `expect()` with a message explaining why the
   code will never panic.
4. New dependencies should be added to the workspace `Cargo.toml` file. This will make it easier to manage dependencies
   and will make sure that all dependencies are of the same version, preventing dependencies being compiled multiple
   times due to version mismatches.
5. If you are adding a new feature that warrants major separation, add it as a new crate and then include it in the
   workspace `Cargo.toml` file. This will make it easier to manage the code and will make sure that the code is well
   separated.
6. Use `cargo clippy` to check for any issues with the code. This will be checked in CI and will cause the build to fail
   if there are any issues. There is no excuse for *your* code to fail the lints.
7. Use `cargo fmt` to format the code. This will be checked in CI and will cause the build to fail if the code is not
   formatted correctly. There is no excuse for *your* code to fail the formatting.
8. Use `#[expect(lint)]` instead of `#[allow(lint)]` if you are sure that the lint is not an issue. This will make it
   easier to find and remove these lints in the future.
9. Use `#[cfg(test)]` to only include code in tests. This will make the code easier to read and understand.
10. Where applicable, add doc strings to functions and modules. This will make it easier for others to understand the
    code.
    Check https://doc.rust-lang.org/nightly/rustdoc/how-to-write-documentation.html for more information on how to write
    good documentation.
11. Unsafe code is ok as long as it is well-documented, and the reason for the unsafe code is explained. If you are not
    sure if the code is safe, ask in the Discord.
12. You will be asked to fix your PR if folders like `.vscode` or `.idea` are included in the PR. These folders are
    specific to your IDE and should not be included in the PR.
13. If you are adding a new feature, make sure to add tests for it. This will make sure that the feature works as
    expected and will help prevent regressions in the future.
14. If you are fixing a bug, make sure to add a test that reproduces the bug. This will make sure that the bug is fixed
    and will help prevent regressions in the future.
15. If your code isn't sufficiently documented, you will be asked to add documentation.
16. If your code doesn't have tests where it should, you will be asked to add tests.
17. Please don't submit massive PRs with 80k changed lines, you will be asked to split these into smaller PRs. It's an
    absolute nightmare to review and verify massive PRs, so please try to keep your PRs small and focused on a single
    feature or bug fix.

## Notes on formatting

Some IDEs have an automatic formatter that will format the code when you save. It is recommended to use this feature to
keep the code formatted correctly.
<br> If you are using VSCode, you can use the `rust-analyzer` extension to format the
code
automatically. This [StackOverflow answer](https://stackoverflow.com/a/67861602/15894829) explains how to set this
up.<br>
If you are using a JetBrains IDE (Intellij, RustRover, CLion, etc.), you can use the `Rust` plugin to format the code
automatically (This plugin is not required for RustRover).
This [Docs page](https://www.jetbrains.com/help/idea/reformat-and-rearrange-code.html#reformat-on-save)
explains how to set this up. Clippy formatting on the fly is recommended as well, though this can cause a noticeable
performance hit.

Automatic formatting is highly recommended as it will ensure that the code you write is correctly formatted as you go,
instead of running `cargo clippy` when you are done and having 400 clippy errors to fix at once. You should still run
the clippy and fmt commands before submitting a PR to make sure that the code is correctly formatted and passes the
lints. However, automatic formatting will help to catch most of these issues as you go.

## Code of Conduct

Please note we have a code of conduct, please follow it in all your interactions with the project.

## License

By contributing, you agree that your contributions will be licensed under the project's license.

### [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
