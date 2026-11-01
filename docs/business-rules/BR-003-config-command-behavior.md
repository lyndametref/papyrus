# BR-003: Papyrus config Command Behavior

## Description

The papyrus config command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → config); behavior not stated there is unspecified.

## Rationale

Based on REQ-01.002 — Read and Edit Project Configuration, this rule exists because the documented config behavior is the contract for papyrus config; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus config` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): config — `README.md` lines 71–93

```text
config

       Read and edit the configuration. Keys use dotted notation for nested values, for example document_types.req.enabled.
       
       papyrus config list
       papyrus config get <key>
       papyrus config set <key> <value>
       papyrus config unset <key>
       
       list
       Print all effective keys, including defaults.

       get <key>
       Print one effective value.

       set <key> <value>
       Persist one value. The value is parsed as a YAML scalar.

       unset <key>
       Remove one key and fall back to its default.

       Precedence is described in papyrus-config(5).
```
