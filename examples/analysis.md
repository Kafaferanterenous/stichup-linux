Analysis of example repos (vectorcraft/pdfcraft/designcraft/wordcraft from storytold)

Key insights relevant to "StichUp - PDF stitch/editor app" with live text editing:

Vectorcraft (text editing - important):
- Rich text editing model: `crates/text/src/edit.rs` has range replace, split/merge style runs, caret/word/para navigation, `replace_range`, `replace_range_styled`, `style_range` - live text editing primitives.
- `edit` module used by engine commands (`text.editRange`, `text.setRangeStyle`) and Type tool.
- Layout + caret: `caret_position`, `hit_byte`, line info for click/drag editing.
- Inline graphics preserved; style runs with splitting/normalization.
- Text object model in `vectorcraft-doc` with runs/styles.

Pdfcraft (PDF text editing + UI):
- `crates/edit/src/text.rs` - in-place text editing on existing PDF lines: reads lines, rewrites content streams (only affected streams), preserves positioning, font handling/substitution logic.
- `crates/ui-egui/src/edit_text_ui.rs` - inline editor UI: preview background (removes source glyphs), draft layout measurement, commit/cancel, visual feedback.
- `crates/engine/src/text_preview.rs` - clone COS, clear block to render background without source glyphs for live preview.
- `crates/edit` provides `text_editor_fonts()`, `register_text_font()`, `text_edit_layout()` for live measurement/preview.
- "Live editing" pattern: show background, overlay textedit box with measured draft rect, live layout check for overflow.

Designcraft/Wordcraft: similar rich-text/edit patterns, doc models.

Recommendations for StichUp:
1. For PDF text editing (live): adopt Pdfcraft's approach - extract text lines/blocks, allow inline edit, rewrite minimal content streams. Reuse preview pattern (render background sans edited glyphs).
2. For rich text UX: borrow Vectorcraft's `text/edit.rs` primitives (range ops, caret navigation, style runs) if we need advanced editing beyond simple line replace.
3. For stitching: implement page merge/reorder/rotate/crop, N-up/booklet-like imposition (Pdfcraft has print/imposition concepts). Also consider combining multiple PDFs into one.
4. UI: egui (used by Pdfcraft ui-egui) - good for Linux desktop.

Start small: basic PDF load (pdfium or pdf crate), page list, reorder/merge, then add inline text edit for simple cases.
