---
id: NFR-05.001
title: "Display the Browser Interface in Named Browsers"
created_date: '2026-11-08T15:56:02Z'
updated_date: '2026-11-16T20:31:52Z'
labels: []
references: []
status: verified
---
# NFR-05.001 Display the Browser Interface in Named Browsers

## Group
NFR-05 Browser Compatibility


## Description
The Papyrus browser interface shall be supported in Firefox version 115 and later. "Supported" means the interface renders and is usable with no layout breakage in Firefox 115 and later.

## Rationale
This records the supported browser family and minimum version without defining visual behavior not supplied in the scope.

## Verification Method
Verify the browser interface using version 115 and later of Firefox, confirming that the interface renders and is usable with no layout breakage in each tested version.

## Change Note
2026-11-16: Multi-browser support (Microsoft Edge 130+, Safari 18+) and the Firefox 100 floor were withdrawn in favour of a Firefox 115 floor for release 1.0. The Mermaid client-side CSSStyleSheet compatibility shim was removed as a consequence.
