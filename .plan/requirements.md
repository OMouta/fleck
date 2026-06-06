# Fleck Requirements

Source: `.plan/spec.md`

Status values: `open`, `partial`, `covered`, `deferred`, `blocked`.

## Product Contract

- REQ-001: Fleck is an open-source, cross-platform, local-first raster image editor for developer asset workflows. Source: sections 1, 2, 18, 20, 22.
- REQ-002: Fleck's differentiator is raster editing with first-class export areas on an infinite workspace. Source: sections 2, 25.
- REQ-003: Fleck must avoid non-goals: Photoshop/Figma/Canva replacement, digital painting suite, RAW editor, video editor, browser-only editor, cloud collaboration tool, AI image generator, or full vector design tool. Source: sections 1, 24.

## Document And Workspace Model

- REQ-004: A `.fleck` workspace stores metadata, canvas settings, layers, placed images, selections, guides, export areas, outputs, recipes, assets, history, and document settings. Source: sections 4.1, 11.1.
- REQ-005: The workspace is infinite or practically unbounded and supports free arrangement of multiple assets, variants, groups, source/processed versions, and export areas. Source: sections 4.2, 6.5.
- REQ-006: Raster layers support editable pixel data, visibility, opacity, lock state, position, bounds, blend mode, alpha, transform, clipping, masks, grouping, merge, duplicate, reorder, hide/show, and rasterization. Source: sections 4.3, 5.3.
- REQ-007: Imported image objects support source tracking, position, scale, rotation, opacity, crop bounds, replacement, duplication, rasterization, and export inclusion. Source: section 4.4.
- REQ-008: Selections support rectangular, elliptical, lasso, polygon, magic wand, color range, expansion, contraction, feathering, inversion, pixel movement, deletion, copy, layer creation, export area creation, and direct export. Source: sections 4.5, 5.2, 7.2.
- REQ-009: Export areas are named rectangular metadata objects with position, size, padding, background, trim, outputs, format, filename, quality, scaling, metadata, path, layer inclusion, and presets. Source: section 4.6.
- REQ-010: Outputs are generated files from export areas and support multiple outputs per area with filename, format, size, scale, quality, compression, background, transparency, metadata, and destination settings. Source: sections 4.7, 7.4.
- REQ-011: Recipes are reusable action/export configurations, command-palette accessible, applicable to layers, images, selections, export areas, or workspace, and saved inside workspaces. Source: section 4.8.
- REQ-012: Workspaces support embedded and linked assets, relinking, replacement, revealing assets, collecting linked assets, and packaging for sharing. Source: section 11.2.
- REQ-013: Workspace file format is portable, versioned, forward-compatible, durable, inspectable where practical, shareable, and suitable for source control where practical. Source: sections 11.1, 11.3.
- REQ-014: Opening older workspace files migrates safely; opening newer files warns when unsupported features exist. Source: section 11.3.

## Editing And Rendering

- REQ-015: Core raster tools include move, selection tools, crop, resize image/canvas, rotate, flips, brush, pencil, eraser, fill, gradient, color picker, clone, healing, blur, sharpen, smudge, text, shape, line, arrow, and rounded rectangle. Source: section 5.1.
- REQ-016: Pixel movement supports selecting, moving, duplicating, deleting, nudging, transforming, copy/paste, paste from clipboard, paste as layer, and paste into selection. Source: section 5.2.
- REQ-017: Text objects support font family, size, weight, color, alignment, line height, letter spacing, editability, rasterization, text box resizing, outlines, and shadows. Source: section 5.4.
- REQ-018: Shape objects support rectangle, rounded rectangle, ellipse, line, arrow, polygon, custom simple path, fill, stroke, stroke width, opacity, corner radius, arrowhead style, alignment, and rasterization. Source: section 5.5.
- REQ-019: Filters and adjustments include brightness, contrast, exposure, saturation, hue, temperature, tint, grayscale, invert, blur, Gaussian blur, sharpen, pixelate, noise, threshold, posterize, outline, shadow, glow, stroke, round corners, and trim transparent pixels. Source: section 5.6.
- REQ-020: Filters apply to layer, selection, export area preview, or whole workspace output. Source: section 5.6.
- REQ-021: Background removal is local-first, accountless, avoids cloud upload, produces editable alpha masks, supports progress, preview/apply/revert, manual cleanup, and optional refinement tools. Source: section 5.7.
- REQ-022: Metadata and cleanup operations include EXIF stripping, color profile remove/preserve, transparency flattening, solid background conversion, empty-pixel trimming, hidden-layer removal on export, PNG optimization, JPEG/WebP compression, palette reduction, and file-size estimation. Source: section 5.8.
- REQ-023: Skia rendering supports workspace compositing, pan/zoom, export overlays, selection outlines, transform handles, checkerboard transparency, pixel grid, guides, shapes, text preview, and high-performance viewport drawing. Source: sections 12.8, 13.

## Canvas Behavior

- REQ-024: Navigation supports smooth pan/zoom, zoom to fit/selection/export area/100%, pixel-perfect zoom, minimap, scroll wheel, trackpad, and keyboard navigation. Source: section 6.1.
- REQ-025: Guides and snapping support rulers, guides, smart guides, pixel/layer/export-area/center/edge/common-size snapping, alignment guides, and spacing guides. Source: section 6.2.
- REQ-026: Pixel grid appears only when useful, aligns to actual pixel boundaries, supports transparency and precise editing, and can be disabled. Source: section 6.3.
- REQ-027: Transparency supports alpha editing, checkerboard preview, transparent/solid export, per-export backgrounds, layer opacity, transparent canvas, and transparent trimming. Source: section 6.4.

## Export System

- REQ-028: Default export uses the smallest rectangle containing visible non-transparent pixels when no export area is selected and none exist, with padding/background/transparency/scale/format/quality/metadata settings. Source: section 7.1.
- REQ-029: Export areas can export individually, in batches, by tag, by folder, by changed state, to clipboard, with preview, and with reveal-in-file-manager. Source: sections 7.3, 7.8.
- REQ-030: Supported export formats include PNG, JPEG, WebP, AVIF, static GIF, BMP, TIFF, ICO, ICNS, rasterized SVG where applicable, PDF where applicable, and clipboard formats including Base64, data URI, Markdown image, and CSS background image. Source: sections 7.5, 16.
- REQ-031: Built-in presets include favicon, `.ico`, PWA, Apple touch, macOS ICNS, Windows/iOS/Android/Electron/Tauri/browser extension icons, GitHub social preview, Open Graph, Twitter/X card, README banner, docs header, and app store sizes where useful. Source: section 7.6.
- REQ-032: Export preview shows final crop, background, padding, transparency, dimensions, format, estimated size, warnings, filename, and destination path. Source: section 7.7.
- REQ-033: Export warnings cover blur risk, empty visible pixels, flattening transparent pixels, filename conflict, target size overflow, and undersized source images. Source: section 7.7.
- REQ-034: Batch export supports selected/all/tag/preset/folder export, remembers location, confirms overwrites, handles conflicts, reports results, and retries failed outputs. Source: section 7.8.
- REQ-035: Fleck CLI uses the same Rust core to export workspaces, export one/all/by tag, validate exports, list areas/outputs, and run recipes. Source: sections 7.9, 12.11.

## Application Interface

- REQ-036: React owns UI while Rust remains document source of truth; TanStack Query coordinates async backend access, and Zustand owns immediate UI state only. Source: sections 12.2, 12.3, 12.6, 12.7, 13.
- REQ-037: Main UI includes central canvas, tool strip, command palette, inspector, layers panel, exports panel, history panel, status bar, top menu, and context menus, with canvas as primary focus. Source: section 9.1.
- REQ-038: Tool strip exposes common tools and is keyboard-accessible. Source: section 9.2.
- REQ-039: Inspector displays properties for layer, image object, selection, export area, text object, shape object, and workspace. Source: section 9.3.
- REQ-040: Layers panel supports list, visibility, locks, drag reorder, grouping, opacity, blend mode, add/delete/duplicate/merge/flatten/rename. Source: section 9.4.
- REQ-041: Exports panel supports listing, renaming, selecting, previewing, adding/removing outputs, duplicating areas, exporting one/all, grouping, tagging, status, and warnings. Source: section 9.5.
- REQ-042: History panel supports viewing operations, undo, redo, jumping where possible, and operation names. Source: section 9.6.
- REQ-043: Context menus expose relevant actions for canvas, layer, selection, export area, image object, output, and panel items without overwhelming users. Source: section 9.7.
- REQ-044: Command palette supports fuzzy search, recent commands, groups, shortcuts, prompts, context-aware ranking, descriptions, aliases, repeat last, history, user commands, and recipe execution. Source: section 8.
- REQ-045: Keyboard model supports command palette, tool shortcuts, nudging, larger nudging, modifier duplicate, spacebar pan, zoom, undo/redo, quick export, export all, rename, delete, and duplicate. Source: section 10.

## Native, Quality, And Operations

- REQ-046: Tauri handles windows, menus, dialogs, filesystem, drag/drop, clipboard, packaging, platform integration, native notifications, and secure UI/Rust bridge. Source: section 12.1.
- REQ-047: Image processing is Rust-native where possible and covers decode, encode, resize, crop, trim, compositing, conversion, color, alpha, compression, and metadata without required external binaries. Source: section 12.9.
- REQ-048: Long operations are non-blocking, show progress where useful, and are cancellable where possible; common editing remains responsive. Source: section 14.
- REQ-049: Color and quality support RGB/alpha editing, profiles where practical, profile export handling, flattening, preservation, high-quality scaling, pixel-perfect scaling, nearest-neighbor, bilinear, bicubic, and Lanczos/equivalent scaling. Source: section 15.
- REQ-050: Clipboard and drag/drop support image paste, selection copy, export result copy, drag image in, drag exported file/output out where supported, and developer copy formats. Source: section 16.
- REQ-051: Settings include theme, accent, default export format/location/padding/background/scaling, transparency grid, shortcuts, command palette, recent files, autosave, file format, background removal, performance, and opt-in telemetry if telemetry exists. Source: section 17.
- REQ-052: Accessibility supports keyboard navigation, accessible menus/dialogs, contrast, scalable text, screen-reader labels where practical, reduced motion, and customizable shortcuts. Source: section 19.
- REQ-053: Cross-platform support includes macOS, Windows, Linux, native dialogs/menus, shortcut conventions, drag/drop, clipboard, open-with, recent files, associations, system theme, and high-DPI. Source: section 20.
- REQ-054: Distribution supports direct download, Homebrew cask, GitHub releases, Windows installer, winget, Linux AppImage, Flatpak, and package-manager naming. Source: section 21.
- REQ-055: Open-source model encourages contributions for presets, recipes, formats, packaging, translations, UI, bugs, plugin commands, docs, and background removal. Source: section 22.
- REQ-056: Extension/plugin system supports commands, recipes, formats, transforms, generators, presets, validators, command palette exposure, clear permissions, local execution, no silent network access, approval for filesystem actions, and stable command API. Source: section 23.
