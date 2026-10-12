# bic Tenant Prompt Chip

Upstream request: [warpdotdev/warp#13176](https://github.com/warpdotdev/warp/issues/13176), closed as a duplicate of [#5084](https://github.com/warpdotdev/warp/issues/5084) (custom context chips).

## Summary

A prompt chip, "bic Tenant", that shows which broadpeak.io tenant the current folder is pinned to and lets the user switch tenant with a click. It is a hardcoded chip for the `bic` CLI (`bpkio-python-tools`), not the general user-authored chip system requested in #5084.

## Problem

`bic` works against one of several tenants. The active tenant for a folder is stored in a `.tenant` file, and the tenant profiles live in `~/.bpkio/tenants`. Users who move between tenants have no at-a-glance indication of which tenant they are in, and switching means remembering the right `bic` command. The same information was previously surfaced through Powerlevel10k custom segments, and its absence is a blocker for moving to Warp.

## Behavior

- The chip shows the first line of the `.tenant` file in the session's working directory.
- The chip is hidden when there is no `.tenant` file in the working directory.
- The chip is orange (`#ff9500`, sampled from the bic logo) and uses a bic logo icon.
- Clicking the chip opens a menu listing every tenant configured on this machine. The current tenant has a checkmark.
- Choosing a tenant writes its label to `.tenant` in the working directory, by running a visible shell command in the session: `printf %s '<label>' > .tenant` (`Set-Content` in PowerShell). The chip updates to the new label.
- If no tenants are configured, the menu says so.
- The chip appears in the prompt editor's list of available chips and is not part of the default prompt.
- The chip is only available when the `TenantChip` feature flag is on.

## Non-goals

- A general mechanism for user-defined chips, a config format, or refresh/timeout settings (tracked in #5084).
- Running `bic` itself, or validating that a chosen tenant has working credentials.
- Remote (SSH) sessions: the tenant list is read from the local machine.

## Open questions

- Whether to list the `[default]` profile as a selectable tenant. It currently is.
- Whether the chip should be in the default prompt.
- Whether the traced icon is legible enough at chip size.
