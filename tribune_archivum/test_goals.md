1. epub_integrity_tests.rs — Basic EPUB file checks

Purpose: Ensure the EPUB is readable and all referenced files exist.

Tests to include:

epub_is_readable — ZIP can be opened.

spine_files_exist — Every SpineItem.Href exists in the ZIP.

toc_exists — toc.ncx exists.

toc_readable_xml — toc.ncx parses without error.

2. toc_structure_tests.rs — TOC / nav structure validation

Purpose: Verify the TOC can be flattened and maps to spine correctly.

Tests to include:

flatten_navpoints — Flatten recursive TOC; ensure no cycles.

toc_entry_maps_to_spine — Each flattened TOC entry maps to exactly one spine file.

no_duplicate_basenames_in_spine — Ensure unique basenames to avoid ambiguous mapping.

fragment_ignored_in_mapping — Confirm #id fragments are ignored when mapping TOC to spine.

playorder_parseable_or_default — PlayOrder is numeric or defaults to 0.

3. pretty_spine_tests.rs — Navigation / PrettySpine expectations

Purpose: Confirm backend-safe navigation can be constructed.

Tests to include:

pretty_spine_index_valid — Each PrettySpineItem.Index corresponds to a valid spine index.

pretty_spine_number_correct — Number = playOrder + 1 (or default 1).

pretty_spine_titles_exist — Title is present (can be empty, just not null).

4. chapter_extraction_tests.rs — Chapter content assumptions

Purpose: Ensure chapters are extractable and valid for backend processing.

Tests to include:

nav_index_in_bounds — navIndex must be within [0, len(e.Nav)).

chapter_file_exists — The spine file for the chapter exists in ZIP.

chapter_html_parseable — HTML parses without error.

chapter_body_exists — <body> element exists.

strip_whitespace_nodes — Whitespace-only text nodes are safely removed.

render_children — Children of <body> are renderable into buffer.

5. contract_invariants_tests.rs — Backend-wide invariants

Purpose: Capture high-level rules that the backend assumes but does not enforce internally.

Tests to include:

no_epub_repair_needed — Backend expects valid EPUB; test that onboarder guarantees this.

flattened_order_is_canonical — Reading order = flattened TOC order.

all_linear_spine_items_in_toc — Spine items that are linear must appear in TOC.

basename_mapping_is_unique — Confirm mapping via basename is unambiguous.

playorder_defaults_applied — Default numeric handling of missing/invalid PlayOrder.

6. How to Use These Tests

Each test file corresponds to a logical test suite in your analyzer / guard dog.

Each test should take a sample EPUB, often with deliberately broken variants, and assert backend contract compliance.

When onboarding, only EPUBs passing all test suites are considered safe for the backend.