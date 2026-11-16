---
id: NFR-04.001
title: "Persist Application Data Only in Text Files"
created_date: '2026-11-08T15:55:54Z'
updated_date: '2026-11-16T20:31:52Z'
labels: []
references: []
status: verified
---
# NFR-04.001 Persist Application Data Only in Text Files

## Group
NFR-04 Persistence


## Description
All data saved by Papyrus shall be stored in text files only. "All data saved by Papyrus" means managed record files (tasks, milestones, releases, decisions, docs) plus configuration; these files shall be UTF-8 text files, with no binary data.

## Rationale
This records the requirement that all saved data use text files without adding a particular text-file format.

## Verification Method
Review Papyrus's saved artifacts and storage mechanisms to confirm that managed record files (tasks, milestones, releases, decisions, docs) and configuration are stored as UTF-8 text files only, with no binary data.
