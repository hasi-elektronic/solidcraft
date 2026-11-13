# HasiSolid / MSSolid validation record

Base: SolidCraft v0.27.1, commit `aa1fe28ddff42cac2990a312765a1a6c50d97dfe`.
The product source is an uncommitted working tree and is not a released build.

## Evidence collected on 2026-10-30

- After the ACP, update, localization and permission changes, source
  `cargo check --locked --offline -p solidcraft` passed separately with the
  `brand-hasisolid` and `brand-mssolid` features. This checks compilation, not
  test execution, installer contents or real desktop behavior.
- `cargo check --locked --offline -p solidcraft-ui-egui --tests` passed for
  each of `brand-hasisolid`, `brand-mssolid` and the unbranded configuration
  on the later working tree. It checks that the test targets compile; it does
  not execute the tests.
- On the HasiSolid test binary, eight targeted tests passed: MCP failure
  localization, permission prompts, product display and canonical labels,
  ACP identity, direct MCP identity, bridge-name summary, disabled updater
  preferences, and localized missing-CLI help. These are in-process tests,
  not real external agent or packaged desktop tests.
- The same HasiSolid binary passed all 22 `update::tests`, all 10 ACP tests,
  all 33 direct API client tests and the pending/complete locale catalog tests.
  These use in-process or simulated services; they do not establish live
  provider connectivity, rendered Windows behavior or installer correctness.
- The MSSolid binary also passed its MCP error test, all 22 updater, all 10 ACP
  and all 33 API client tests, plus permission, bridge-summary and missing-CLI
  help tests. This binary predates the later label-precedence source fix.
- On the rebuilt MSSolid UI test binary after the label fix, all seven
  `companion::tests` passed, including real STEP export/reopen, source overwrite
  prevention, document change, point-cloud rejection and invalid assembly cases.
  In `companion_publish::windows::tests`, three local tests passed and four
  junction/UNC/SMB fixture-dependent tests were ignored. These tests do not
  exercise the real Viewer launch or a packaged desktop application.
- A broader MSSolid agent-chat run on that binary passed 81 of 88 tests. Seven
  GUI harness tests failed at English widget expectations because branded
  defaults select German. Those seven tests now explicitly choose English
  before opening chat; a separate branded test checks the German default and
  displayed send button. Independent QA reviewed these changes and HasiSolid
  test targets passed `cargo check` on the updated source. A rebuilt HasiSolid
  binary passed all 89 chat tests, including the changed tests and the German
  default test. The rebuilt MSSolid binary also passed all 89 chat tests.
  The German default test does not click the button.
- Broader HasiSolid and MSSolid `i18n::tests` runs exposed an alias-selection
  regression as newly translated render labels hid established engine labels.
  The label lookup now prefers available non-render translations for those
  two known labels, while other labels keep their existing precedence and
  direct render IDs keep their own translations. The exact updated Rust
  function passed a standalone harness for Appearance, Checkpoint, Ground,
  render-only fallback and unknown labels. The rebuilt MSSolid binary passed
  all 81 `i18n::tests`, 22 updater tests, 10 ACP tests, 33 API client tests,
  plus four targeted permission, MCP error, bridge and help tests. The rebuilt
  HasiSolid binary also passed all 81 i18n tests, seven STEP transfer tests,
  22 updater tests, 10 ACP tests and 33 API client tests.
- All 12 `i18n_ui_tests` passed on the HasiSolid and MSSolid binaries,
  including German dialogs, language-preference cancellation and translated
  button layout. The MSSolid binary predates the latest chat-test-only changes.
  These are egui harness tests, not a native Windows package exercise.
- After the later branding changes, the exact `product_display` function was
  extracted into a standalone Rust harness with a minimal product stub. Its
  brand, MCP identity, unchanged attribution and lock-owner checks passed for
  HasiSolid, MSSolid and unbranded configuration. This is not an application
  build or a rendered UI test.
- A separate merge of the seven complete locale catalogs confirmed zero missing
  keys and zero named placeholder-set mismatches after adding the CLI error.
- ACP client identities, the session MCP bridge, system prompts and direct API
  MCP client identity now use the selected product. Independent reviewers
  inspected the source. The new tests are not yet run; real agent/provider
  sessions, same-name user MCP configuration and simultaneous brands are not
  validated. Existing explicit always-allow entries are not migrated.
- Independent code review identified an ACP permission bypass based on a
  tool-title substring. The automatic branch was removed and a regression test
  was added; review of the correction found no remaining P1 in that narrow
  scope. The test and real agent flow have not yet been run.
- Branded preferences now explain that the product's own packages provide
  updates. A separate branded test exercises the preferences row and checks
  that automatic, manual and install jobs do not start. Independent reviewers
  examined the source and test; the test has not yet run. Web preferences keep
  their original reload explanation, and upstream updater tests remain enabled
  for unbranded builds.
- Product identity unit test passed for unbranded, HasiSolid and MSSolid builds.
- HasiSolid engine brand-specific AI setup tests passed: three tests, including
  preservation of other product entries and rejection of live setup.
- Before integrating the publication helper, HasiSolid desktop `cargo check --locked --offline` passed again after an
  unused upstream icon helper was restricted to unbranded builds; the second
  run reported no warnings.
- Before that integration, MSSolid desktop `cargo check --locked --offline`
  also passed without warnings. These earlier checks cover the pre-integration tree.
- After the publication, point-cloud and assembly changes, HasiSolid desktop
  `cargo check --locked --offline` passed without warnings. The equivalent
  MSSolid desktop check also passed after the temporary disk-space interruption.
  Subsequent CLI and installer changes are outside those desktop checks.
- After the CLI branding and icon changes, HasiSolid and MSSolid CLI
  `cargo check --locked --offline` both passed. The packaged CLI and complete
  CLI test suite remain unverified.
- The real CLI integration test `branded_help_and_icon_use_product_identity`
  subsequently passed once for HasiSolid and once for MSSolid. These are
  validation builds, not the packaged CLI executables.
- The current CLI help-branding function was extracted into a minimal Rust
  harness and passed checks for HasiSolid, MSSolid and unbranded identity. This
  does not run the actual CLI executable or validate an icon.
- A real HasiSolid CLI integration-test build was started with its build and
  temporary directories on E:, but was interrupted during dependency
  compilation while C: had approximately 2 GB free. No test result is claimed.
- Inno Setup 6.7.3 accepted `brands.iss` for both brands with compiler output
  disabled. The staging executable was a renamed Windows utility used solely as
  a syntax fixture; no application setup package or installation was tested.
  The installed Inno Setup licence permits commercial use, while its publisher
  requests a commercial licence purchase and states it is not strictly required:
  https://jrsoftware.org/isorder.php
- A Windows NTFS filesystem experiment used a disposable directory, a Unicode
  filename and synthetic text. While the staging file was open with read sharing
  only, attempts to write and delete it were denied. Creating a hard link,
  closing the handle, cleaning the staging name and reading the result passed.
- A separate disposable test on `\\sickserver\Allgemein` passed protected
  writing, hard-link creation, cleanup and content verification over UNC. The
  test used synthetic text, not a CAD model. These results apply only to the
  tested filesystems and share configuration.
- The integrated `companion_publish.rs` (SHA-256
  `25920e540302e0e3483e7dca8209c1ed0491e3489447dd5daf8c13ae7bcd1e0d`)
  was imported directly into a separate test crate. Seven tests passed: local,
  invalid-path, long Unicode disk, normal and verbatim UNC (including >260
  UTF-16 units at `\\sickserver\Allgemein`) and junction rejection. The
  standalone harness emitted unused-code warnings because it does not contain
  the application call site. The tests used synthetic bytes, not real STEP
  geometry, and did not start the Viewer.
- The family-transfer catalog has 21 keys in each of eight languages, with
  matching interpolation variables. This is a structural check, not a full
  linguistic or rendered-interface review.
- Merging the base and pending English and German JSON catalogs yielded 4,713
  English keys and 4,713 German keys, with no missing German keys or named
  placeholder-set mismatches. This does not check translation accuracy, hardcoded
  UI text, or actual rendered screens.
- The current `component_depth` function and its shared-assembly, cycle and
  64/65-depth test were extracted into a standalone Rust harness with a minimal
  product-type stub and passed. This checks the algorithm only, not STEP
  serialization, application integration, or a large real assembly.
- `cargo fmt --all -- --check` passed after the extended-path test additions.
- `cargo fmt --all -- --check` and `git diff --check` passed on the later
  working tree; Git reported line-ending conversion warnings.
- The current UI test build and queued full-tree checks were interrupted before
  completion after free space on `C:` fell to approximately 250 MB. No result
  is claimed for those interrupted runs. Build artifacts remain under the
  separate `E:\Kunden\Sickinger\.build` validation directory.

## Outstanding release gates

Hard-link support is not universal. A failed link must remain a reported error;
the current implementation cannot claim support for every writable filesystem
or SMB configuration. The complete application needs native STEP geometry,
assembly placement, units, no-overwrite races, failure and cancellation tests,
followed by independent source and actual package review. Installation, upgrade,
removal, coexistence, signing, and the exact clean-source release build have not
been validated. Validation artifacts must not be described as released packages.
The installer's reparse-point directory checks do not close a concurrent
same-user replacement race between checking the path and writing files; the
independent code reviewer identified this as a P2 issue for production release.
Holding verified directory handles can reduce this risk, but does not protect
the first-create/open interval or individual path-based file operations. The
reviewer therefore did not accept it as a complete mitigation. A native,
handle-relative publishing design and its rollback behavior need evaluation.
Microsoft documents handle-relative rename targets in `FILE_RENAME_INFO`:
https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info
Opening the final reparse point itself does not establish safety of preceding
path components; directory creation, replacement, rollback and removal still
need a coherent, independently reviewed implementation and adversarial tests.
An independent review also considered `cap-std` directory capabilities.
They require a securely opened Windows root handle and do not by themselves
provide no-replace publication or atomic recursive removal; Inno's own
uninstaller and log writes remain in scope. No dependency was added.
The English and German catalog display values containing the upstream product
name are now covered by selected product-display overrides, including the
separately handled lock-owner message. This is a static catalog check; other
hardcoded UI, protocol identifiers and upstream attribution still need review.
The German catalog also requires a linguistic and native UI review before it
can be called complete.

The independent QA and release reviewers encountered an account usage limit on
2026-10-30. Their final pre-build review of the current tree is outstanding;
earlier partial reports are not approvals for subsequent source changes.

Point clouds are rejected before STEP transfer so they cannot be silently
omitted. Invalid assembly references and bodies unreachable from the root are
also rejected. On the current kernel source, both `solidcraft-io` tests for
invalid references, unreachable bodies, and nested/repeated occurrences passed.
The UI companion tests on both brands also passed point-cloud rejection and
valid root-body STEP export/reopen. Real Viewer launch and measurement, and
larger production assemblies, remain unvalidated.

## Clean-room incident

During investigation of Windows file publishing, an external GPL-licensed source
page (`oc-rsync`, `fast_io/win_atomic_commit.rs`) was opened on 2026-10-30 before its licence was checked, contrary to the upstream
clean-room instruction. No dependency on that project was added and no code was
intentionally copied from it. An independent release reviewer found no matching
project names in the manifests, lockfile or attribution records; this does not
prove provenance or legal compliance. The prior hard-link approach existed before
the source page was opened, but it was subsequently restored and adjusted; no
pre-access file hash is recorded. A reviewer who has not viewed that source must
independently assess the affected implementation using this project's own code
and permitted platform documentation. Dependency and packaged-content licence
checks remain required and do not erase the source-access incident.

The Windows publication helper was independently drafted from the requirements,
without reading the prior `companion.rs` implementation or the GPL source, by a
separate agent. The draft reviewed before integration had SHA-256
`38b8203f1637656fa57ce740fc895a6279d0b230136409e26af998dacad89870`.
The same independent author later prepared the extended-path draft with SHA-256
`9bb8d35560e2feab78d5972ec88a84600777d745b62f52150be5da32a36a6c0f`;
the integrated file differs in formatting, visibility and test additions.
It is project-authored code intended to be distributed under the workspace's
`MIT OR Apache-2.0` licence. Its origin is based on the agent's recorded report;
this does not independently prove absence of influence. The integrated source,
call site, test results and distribution contents still require independent review.
