# NFR-04.001 Persist Application Data Only in Text Files

## Description
Persisted Papyrus application data shall use text files only; no other persistence mechanism shall be used.

## Rationale
This records the requested text-file-only persistence constraint without deciding which files or data count as application data.

## Verification Method
Review the application's persistence paths and stored formats to confirm that persisted application data is held only in text files and that no other persistence mechanism is used. The boundary of application data and what qualifies as a text file remain unspecified.
