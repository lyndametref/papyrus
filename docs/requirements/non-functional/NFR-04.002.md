# NFR-04.002 Restrict File Access to the Project Directory

## Description
Papyrus shall not read from or write to paths outside the project directory, except during `init`. The project directory boundary shall be enforced by canonicalizing paths and resolving symbolic links, rejecting any path that escapes the canonical project root. The `init` exception permits writing outside the project directory only to create the project directory itself.

## Rationale
This records the requested boundary on Papyrus file access and the stated `init` exception.

## Verification Method
Exercise Papyrus operations, including `init`, while monitoring filesystem reads and writes; confirm that no operation other than `init` reads from or writes to a path outside the project directory, that symlink traversal cannot escape the canonical project root, and that `init` writes outside the project directory only to create the project directory itself.
