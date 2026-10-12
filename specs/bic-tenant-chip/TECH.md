# bic Tenant Prompt Chip: Tech Spec

See [PRODUCT.md](./PRODUCT.md) for behavior.

## Relevant code

- `crates/warp_features/src/lib.rs`: `FeatureFlag::TenantChip` (in `DOGFOOD_FLAGS`).
- `app/Cargo.toml`, `app/src/features.rs`: the `tenant_chip` Cargo feature and its mapping to the flag.
- `app/src/context_chips/mod.rs`: `ContextChipKind::Tenant`, its `to_chip` arm, placeholder value, color (`BIC_ORANGE`), `udi_icon`, and the `available_chips` entry. Both the chip and the entry are gated on the flag.
- `app/src/context_chips/builtins.rs`: `tenant_label()`, the per-shell command that prints the first line of `.tenant`.
- `app/src/context_chips/tenant_popup.rs`: `TenantPopupView`, modelled on `NodeVersionPopupView`, plus tenant-list parsing.
- `app/src/context_chips/tenant_popup_tests.rs`: parsing tests.
- `app/src/context_chips/display_chip.rs`: `DisplayChipKind::Tenant`, chip rendering, toggle/close handling, and `PromptChipShellCommand::SwitchTenant`.
- `app/src/terminal/input.rs`: renders `SwitchTenant` into a shell command.
- `app/assets/bundled/svg/bic-logo.svg`, `crates/warp_core/src/ui/icons.rs`: `Icon::Bic`.

## Design

**Display.** The chip is a `shell_builtin` with `RefreshConfig::OnDemandOnly`, the same as the Kubernetes chip. The command runs in the session's shell, so `.tenant` is resolved against the session's working directory. A failing command yields no value, which hides the chip.

**Tenant list.** `list_tenants()` reads `$BPKIO_HOME/tenants`, falling back to `~/.bpkio/tenants`, mirroring `_get_bpkio_home()` in the Python SDK. `parse_tenant_labels` returns only `[section]` names. The file also contains API keys and passwords, so no values are read, retained or logged. The list is read each time the popup is created.

**Switching.** Selecting a row emits `PromptChipShellCommand::SwitchTenant`, which is typed and run in the session like `NvmUse`. Labels are quoted with `shell_quote_arg`. The write happens in the shell, so it is visible to the user and respects the session's cwd and permissions.

**Icon.** `bic-logo.svg` was traced from `bpkio-cli/assets/logo_col1.png` in `bpkio-python-tools` with potrace (a single even-odd path in a 24x24 viewBox) so the chip color tints it.

## Testing

- Unit tests cover `parse_tenant_labels` (header extraction, indented lines, blank and bracket-only lines, comments, empty file).
- Not covered by automated tests: chip rendering, the popup, and the switch flow. These need a manual check in the GUI with `cargo run --features tenant_chip`.

## Follow-ups

- Fold this into the generic custom-chip design when #5084 is implemented.
- Decide on default-prompt inclusion and promotion beyond dogfood.
