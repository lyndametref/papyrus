# NFR-04.001 Persist Application Data Only in Text Files

## Description
All data saved by Papyrus shall be stored in text files only. "All data saved by Papyrus" means managed record files (tasks, milestones, releases, decisions, docs) plus configuration; these files shall be UTF-8 text files, with no binary data.

## Rationale
This records the requirement that all saved data use text files without adding a particular text-file format.

## Verification Method
Review Papyrus's saved artifacts and storage mechanisms to confirm that managed record files (tasks, milestones, releases, decisions, docs) and configuration are stored as UTF-8 text files only, with no binary data.
