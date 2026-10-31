# NFR-04.002 Restrict File Access to the Project Directory

## Description
Papyrus shall not read from or write to paths outside the project directory, except during `init`.

## Rationale
This records the requested boundary on Papyrus file access and the stated `init` exception.

## Verification Method
Exercise Papyrus operations, including `init`, while monitoring filesystem reads and writes; confirm that no operation other than `init` reads from or writes to a path outside the project directory.

## Open Questions
The exact boundary of the project directory (including how symbolic links are treated) and the scope of the `init` exception have not been specified.
