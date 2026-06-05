# Fleck Specification

## 1. Product Definition

Fleck is an open-source, cross-platform raster image editor built around an infinite workspace, command palette, and first-class export areas.

It combines traditional pixel editing with developer-focused asset workflows. Users can edit images manually, arrange multiple assets on a workspace, define named export regions, and export exactly what they need in multiple formats and sizes.

Fleck is designed for practical image work: logos, icons, screenshots, app assets, website media, favicons, Open Graph images, documentation graphics, and small photo edits.

Fleck is not a design suite, drawing app, or AI image generator. It is a focused raster editor for people who want to edit pixels and produce usable assets quickly.

## 2. Product Positioning

Fleck is a command-palette-first raster editor for developers and makers.

It sits between simple paint apps, heavyweight photo editors, and asset conversion tools.

It provides:

* manual raster editing
* infinite workspace organization
* named export areas
* reusable asset workspaces
* developer-focused export presets
* fast command-driven actions
* local-first editing
* open-source transparency

The core product idea is:

Edit pixels freely. Mark export areas. Export exactly what you need.

## 3. Target Users

Fleck is built for:

* software developers
* indie hackers
* product engineers
* open-source maintainers
* technical founders
* UI builders
* people maintaining websites, apps, and documentation
* people who need simple image editing without opening a heavy creative suite

Typical use cases include:

* cleaning up a logo
* removing a background
* resizing an icon into multiple sizes
* creating favicon files
* exporting `.ico` files
* generating app icon sets
* making Open Graph images
* preparing README images
* cropping screenshots
* moving pixels around
* editing transparent PNGs
* exporting light and dark logo variants
* preparing assets for a repo

## 4. Core Concepts

### 4.1 Workspace

A Fleck document is a workspace.

A workspace is an infinite or practically unbounded 2D canvas where users can place, edit, arrange, and export raster assets.

Unlike traditional image editors, the workspace itself is not necessarily the export. It is a working area that can contain multiple assets, drafts, variants, and export areas.

A workspace contains:

* raster layers
* image objects
* selections
* guides
* export areas
* output presets
* workspace metadata
* source images
* editing history
* document settings

The workspace can be saved as a `.fleck` file and shared with others.

### 4.2 Infinite Canvas

Fleck uses an infinite-canvas model similar to vector and design tools.

Users can pan, zoom, arrange images freely, create multiple asset variants, and keep related work together.

The canvas supports:

* free placement of images
* grouped assets
* export regions
* zooming
* panning
* pixel grid view
* transparency checkerboard
* snapping
* guides
* rulers
* alignment helpers

The infinite canvas should not make simple workflows slower. A user can still open a single image and export it like a normal raster editor.

### 4.3 Raster Layers

Raster layers contain editable pixel data.

Each layer supports:

* name
* visibility
* opacity
* lock state
* position
* bounds
* blend mode
* alpha channel
* transform
* clipping behavior
* selection interaction
* export participation

Layers can be moved, reordered, duplicated, merged, hidden, locked, grouped, and rasterized.

### 4.4 Image Objects

Imported images can exist as placed objects on the workspace.

An image object may reference an embedded source image and expose raster-editing behavior when edited.

Image objects support:

* position
* scale
* rotation
* opacity
* crop bounds
* original source tracking
* rasterization
* replacement
* duplication
* export inclusion

### 4.5 Selection

Selections define an editable region of pixels.

Selections support:

* rectangular selection
* elliptical selection
* lasso selection
* polygon selection
* magic wand selection
* color range selection
* selection expansion
* selection contraction
* feathering
* inversion
* moving selected pixels
* deleting selected pixels
* copying selected pixels
* creating a new layer from selection
* creating an export area from selection
* exporting selection directly

Selections are temporary editing regions, not permanent document objects unless converted into layers, masks, or export areas.

### 4.6 Export Areas

Export areas are first-class document objects.

An export area is a named rectangular region on the workspace that defines an output target.

Export areas are not pixels. They are metadata over the workspace.

An export area contains:

* name
* position
* width
* height
* padding
* background behavior
* trim behavior
* output format
* output filename
* quality settings
* scaling rules
* size variants
* metadata behavior
* export path
* included/excluded layers
* output presets

Export areas appear visually on the workspace with a distinct frame. The frame should clearly communicate that the region is an export boundary, not image content.

Suggested visual treatment:

* hatched or crossed border
* label with name
* size indicator
* output format badges
* resize handles
* subtle selected state
* preview mode that dims outside the area

Example export areas:

* `favicon`
* `app-icon`
* `logo-light`
* `logo-dark`
* `og-image`
* `readme-banner`
* `twitter-card`
* `docs-header`

### 4.7 Outputs

An output is a generated file produced from an export area.

One export area can produce one or many outputs.

Example:

Export area: `favicon`

Outputs:

* `favicon.ico`
* `icon-16.png`
* `icon-32.png`
* `icon-192.png`
* `icon-512.png`
* `apple-touch-icon.png`

Example:

Export area: `og-image`

Outputs:

* `og-image.png`
* `og-image.webp`

Each output can define:

* filename
* format
* width
* height
* scale
* quality
* compression settings
* background
* transparency behavior
* metadata stripping
* destination folder

### 4.8 Recipes

Recipes are reusable action or export configurations.

A recipe can be applied to:

* a layer
* a selected image
* a selection
* an export area
* the whole workspace

Recipe examples:

* Generate favicon pack
* Generate app icon set
* Export web media pack
* Create Open Graph image
* Remove background and trim
* Make square with padding
* Compress under target size
* Export transparent logo variants
* Generate README image
* Create `.ico` file
* Create `.icns` file

Recipes are command-palette accessible and can be saved inside a workspace.

## 5. Editing Features

### 5.1 Core Raster Editing

Fleck supports practical raster editing tools:

* move tool
* rectangular selection
* elliptical selection
* lasso selection
* magic wand selection
* crop
* resize image
* resize canvas
* rotate
* flip horizontal
* flip vertical
* paintbrush
* pencil
* eraser
* fill bucket
* gradient fill
* color picker
* clone tool
* healing tool
* blur tool
* sharpen tool
* smudge tool
* text tool
* shape tool
* line tool
* arrow tool
* rounded rectangle tool

### 5.2 Pixel Movement

Fleck allows direct pixel manipulation.

Users can:

* select pixels
* move selected pixels
* duplicate selected pixels
* delete selected pixels
* nudge pixels with arrow keys
* move pixels to a new layer
* transform selected pixels
* copy/paste pixels
* paste images from clipboard
* paste as new layer
* paste into selection

This is important because Fleck is not only an asset exporter. It is also a real raster editor.

### 5.3 Layers

Fleck supports layered editing.

Layer features:

* create layer
* delete layer
* duplicate layer
* rename layer
* reorder layer
* group layers
* merge layers
* flatten image
* hide/show layer
* lock layer
* opacity control
* blend modes
* clipping
* masks
* rasterize object
* layer bounds
* trim layer to visible pixels

Blend modes include:

* normal
* multiply
* screen
* overlay
* darken
* lighten
* color dodge
* color burn
* hard light
* soft light
* difference
* exclusion
* hue
* saturation
* color
* luminosity

### 5.4 Text

Fleck includes a practical text tool.

Text supports:

* font family
* font size
* font weight
* color
* alignment
* line height
* letter spacing
* editable text objects
* rasterization
* text box resizing
* basic text outlines
* basic shadows

Text is designed for asset labels, social previews, screenshots, and simple graphic composition.

### 5.5 Shapes

Fleck supports basic shape creation.

Shapes include:

* rectangle
* rounded rectangle
* ellipse
* line
* arrow
* polygon
* custom simple path

Shape properties include:

* fill
* stroke
* stroke width
* opacity
* corner radius
* arrowhead style
* alignment
* rasterization

Shapes are useful for screenshot annotation, simple media graphics, and icon composition.

### 5.6 Filters and Adjustments

Fleck includes common practical filters and adjustments:

* brightness
* contrast
* exposure
* saturation
* hue
* temperature
* tint
* grayscale
* invert
* blur
* Gaussian blur
* sharpen
* pixelate
* noise
* threshold
* posterize
* outline
* shadow
* glow
* stroke
* round corners
* trim transparent pixels

Filters can be applied to:

* layer
* selection
* export area preview
* whole workspace output

### 5.7 Background Removal

Fleck supports local-first background removal.

Background removal can be applied to:

* selected layer
* selected image object
* selection
* pasted image

Background removal outputs an alpha mask and preserves the result as editable raster data.

Background removal behavior:

* runs locally when possible
* does not require an account
* does not require cloud upload
* shows progress for long operations
* supports applying, previewing, and reverting
* allows manual cleanup after removal

Optional refinement tools:

* refine edge
* recover foreground
* erase background
* smooth mask
* feather mask
* expand/contract mask

### 5.8 Metadata and Cleanup

Fleck supports image cleanup operations:

* strip EXIF metadata
* remove color profile
* preserve color profile
* flatten transparency
* convert transparent background to solid color
* trim empty pixels
* remove hidden layers on export
* optimize PNG
* compress JPEG/WebP
* reduce color palette
* estimate output file size

## 6. Workspace and Canvas Behavior

### 6.1 Navigation

The workspace supports:

* smooth pan
* smooth zoom
* zoom to fit
* zoom to selection
* zoom to export area
* zoom to 100%
* pixel-perfect zoom
* minimap
* scroll-wheel navigation
* trackpad navigation
* keyboard navigation

### 6.2 Guides and Snapping

Fleck supports layout helpers:

* rulers
* guides
* smart guides
* snapping to pixels
* snapping to layer bounds
* snapping to export area bounds
* snapping to center
* snapping to edges
* snapping to common sizes
* alignment guides
* spacing guides

### 6.3 Pixel Grid

At high zoom levels, Fleck displays a pixel grid.

Pixel grid behavior:

* appears only when useful
* aligns to actual pixel boundaries
* supports transparent images
* supports precise pixel editing
* can be disabled

### 6.4 Transparency

Transparency is a core feature.

Fleck supports:

* alpha channel editing
* checkerboard transparency preview
* transparent export
* solid background export
* per-export background settings
* layer opacity
* transparent canvas
* transparent trimming

### 6.5 Workspace Organization

Users can organize assets spatially.

Features:

* place multiple images
* arrange variants
* group related assets
* rename objects
* add notes or labels
* use export areas as named output frames
* duplicate asset groups
* keep source and processed versions together

The workspace should feel useful as a shareable project file.

## 7. Export System

### 7.1 Default Export Behavior

If no export area is selected and no export areas exist, Fleck exports the smallest rectangle containing visible non-transparent pixels.

This makes Fleck behave like a normal raster editor by default.

Default export can include:

* padding
* background color
* transparency
* scale
* format
* quality
* metadata behavior

### 7.2 Selection Export

Any active selection can be exported directly.

Selection export supports:

* export selection as PNG/JPEG/WebP/ICO
* copy selection to clipboard
* create export area from selection
* export with padding
* export with background
* export at multiple scales

### 7.3 Export Area Export

Export areas can be exported individually or in batches.

Export area actions:

* export selected area
* export all areas
* export areas by tag
* export areas by folder
* export changed areas
* copy export result to clipboard
* preview export result
* reveal exported file in system file manager

### 7.4 Multiple Outputs Per Export Area

Each export area can produce multiple files.

Output settings include:

* filename
* folder
* format
* scale
* width
* height
* quality
* compression
* background
* transparency
* metadata stripping

This allows one area to generate complete asset packs.

### 7.5 Supported Export Formats

Fleck supports common developer and web formats:

* PNG
* JPEG
* WebP
* AVIF
* GIF static export
* BMP
* TIFF
* ICO
* ICNS
* SVG rasterized export where applicable
* PDF export where applicable

Fleck also supports clipboard exports:

* copy PNG
* copy JPEG
* copy WebP
* copy as Base64
* copy as data URI
* copy as Markdown image
* copy as CSS background image

### 7.6 Icon and Asset Presets

Fleck includes built-in export presets for common developer workflows.

Presets include:

* favicon pack
* `.ico` file
* PWA icon pack
* Apple touch icon
* macOS `.icns`
* Windows app icon set
* iOS app icon set
* Android icon set
* Electron app icon set
* Tauri app icon set
* browser extension icon set
* GitHub social preview
* Open Graph image
* Twitter/X card image
* README banner
* documentation header
* app store image sizes where useful

### 7.7 Export Preview

Each export area has a preview.

Preview shows:

* final crop
* background
* padding
* transparency
* output dimensions
* format
* estimated file size
* warnings
* filename
* destination path

Warnings include:

* output may be blurry
* export area contains no visible pixels
* transparent pixels will be flattened
* filename conflict
* output exceeds target file size
* source image too small for requested size

### 7.8 Batch Export

Fleck supports exporting multiple outputs at once.

Batch export supports:

* export all areas
* export selected areas
* export by tag
* export by preset
* export to selected folder
* remember export location
* overwrite confirmation
* conflict handling
* export report
* failed-output retry

### 7.9 Export Automation

Fleck workspaces can be exported outside the GUI through the Fleck CLI.

The CLI supports:

* exporting all areas
* exporting one area
* exporting by tag
* exporting to a folder
* validating workspace exports
* listing export areas
* listing outputs
* running export recipes

The CLI uses the same core engine as the desktop app.

## 8. Command Palette

The command palette is a primary interface.

It supports:

* fuzzy command search
* recent commands
* command groups
* keyboard shortcuts
* parameter prompts
* context-aware commands
* command descriptions
* command aliases
* repeat last command
* command history
* user-defined commands
* recipe execution

Examples:

* Open Image
* Save Workspace
* Export All Areas
* Export Selected Area
* Create Export Area from Selection
* Rename Export Area
* Generate Favicon Pack
* Generate App Icons
* Resize Image
* Resize Canvas
* Trim Transparent Pixels
* Remove Background
* Add Padding
* Set Format to WebP
* Copy as Base64
* Copy as Markdown Image
* Toggle Pixel Grid
* Zoom to Export Area
* Move Selection to New Layer

Commands can be context-aware.

When a layer is selected, layer commands rank higher.

When an export area is selected, export commands rank higher.

When a selection exists, selection commands rank higher.

## 9. Interface

### 9.1 Main Layout

The default interface contains:

* central workspace canvas
* tool strip
* command palette
* inspector panel
* layers panel
* exports panel
* history panel
* status bar
* top menu
* context menus

The canvas is always the primary focus.

### 9.2 Tool Strip

The tool strip contains common editing tools:

* move
* select
* lasso
* magic wand
* brush
* eraser
* fill
* crop
* text
* shape
* color picker
* export area tool
* hand/pan
* zoom

Tools should be keyboard-accessible.

### 9.3 Inspector

The inspector shows properties for the current selection.

It can inspect:

* layer
* image object
* selection
* export area
* text object
* shape object
* workspace

For export areas, the inspector shows:

* name
* dimensions
* position
* padding
* background
* outputs
* format
* quality
* scale
* destination
* preview

For layers, the inspector shows:

* name
* visibility
* opacity
* blend mode
* position
* size
* lock state
* effects
* masks

### 9.4 Layers Panel

The layers panel supports:

* layer list
* visibility toggles
* lock toggles
* drag reorder
* grouping
* opacity
* blend mode
* add layer
* delete layer
* duplicate layer
* merge layer
* flatten image
* rename layer

### 9.5 Exports Panel

The exports panel shows all export areas and outputs.

It supports:

* list export areas
* rename export areas
* select export area
* preview export area
* add output
* remove output
* duplicate export area
* export one area
* export all areas
* group export areas
* tag export areas
* show export status
* show warnings

### 9.6 History Panel

The history panel shows undoable operations.

It supports:

* view past operations
* undo
* redo
* jump to previous state where possible
* inspect operation names

### 9.7 Context Menus

Right-click menus are available for:

* canvas
* layer
* selection
* export area
* image object
* output
* panel items

Context menus should expose relevant actions without overwhelming the user.

## 10. Keyboard and Interaction Model

Fleck is keyboard-friendly.

Core interactions:

* command palette shortcut
* tool shortcuts
* arrow-key nudging
* shift for larger nudging
* modifier-based duplicate
* spacebar pan
* common zoom shortcuts
* common undo/redo shortcuts
* quick export shortcut
* export all shortcut
* rename shortcut
* delete shortcut
* duplicate shortcut

Fleck should feel natural to people used to developer tools, vector editors, and traditional raster editors.

## 11. File Format

### 11.1 `.fleck` Workspace

A `.fleck` file stores the entire workspace.

It includes:

* workspace metadata
* canvas settings
* layers
* placed images
* export areas
* output definitions
* recipes
* guides
* groups
* document settings
* embedded assets or references
* color profiles where applicable
* edit history where applicable

The format should be:

* portable
* versioned
* forward-compatible
* durable
* inspectable where practical
* suitable for sharing
* suitable for source control where practical

### 11.2 Embedded and Linked Assets

Fleck supports embedded assets and linked assets.

Embedded assets are stored inside the `.fleck` workspace.

Linked assets reference files on disk.

Users can:

* embed linked asset
* relink missing asset
* replace asset
* reveal asset
* collect all linked assets into workspace
* package workspace for sharing

### 11.3 Versioning and Compatibility

Fleck documents include a format version.

When opening older files, Fleck migrates them safely.

When opening newer files, Fleck warns the user if unsupported features exist.

## 12. Stack

### 12.1 Tauri

Tauri is the desktop application shell.

It handles:

* native windows
* menus
* file dialogs
* filesystem access
* drag and drop
* clipboard
* app packaging
* platform integration
* native notifications
* secure bridge between UI and Rust

Tauri does not own the document model or rendering model.

### 12.2 Rust

Rust owns the core application engine.

Rust handles:

* document model
* workspace model
* layer model
* export area model
* command execution
* undo/redo system
* file format
* image operations
* export pipeline
* background jobs
* CLI integration
* performance-critical logic

Rust is the source of truth for the Fleck workspace.

### 12.3 React

React owns the interface.

React handles:

* panels
* toolbars
* inspectors
* command palette
* dialogs
* menus
* toasts
* settings screens
* export configuration UI
* layer list UI
* export list UI
* workspace metadata display

React does not own raw pixel data or the authoritative document model.

### 12.4 Tailwind CSS

Tailwind CSS handles styling.

It is used for:

* layout
* spacing
* typography
* borders
* color tokens
* panel styling
* toolbar styling
* component variants
* dark mode
* responsive desktop layouts

Tailwind is used as a build-time styling system.

### 12.5 shadcn/ui and Radix UI

shadcn/ui provides a polished component foundation.

Radix UI provides accessible primitives.

Used for:

* buttons
* inputs
* sliders
* dialogs
* dropdowns
* menus
* context menus
* popovers
* tabs
* tooltips
* command palette shell
* resizable panels
* toggles
* toasts

Components are customized so Fleck feels like a desktop editor, not a generic web dashboard.

### 12.6 TanStack Query

TanStack Query manages async state between React and the Rust backend.

It handles:

* loading workspace metadata
* loading previews
* invoking long-running operations
* tracking export jobs
* tracking background-removal jobs
* refreshing derived data
* caching expensive read operations
* handling mutation states
* retrying recoverable operations
* invalidating stale UI data

TanStack Query does not own the document model. It coordinates async access to Rust-owned state.

### 12.7 Zustand

Zustand manages immediate UI state.

It handles:

* selected tool
* selected layer ID
* selected export area ID
* active panel
* command palette open state
* hovered object
* temporary drag state
* viewport UI state
* inspector tab
* visible overlays
* local preferences

Zustand is for local UI interaction state, not document truth.

### 12.8 Skia

Skia is the rendering engine.

It handles:

* canvas rendering
* raster compositing preview
* zooming and panning
* export area overlays
* selection outlines
* transform handles
* checkerboard transparency
* pixel grid
* guides
* shape rendering
* text rendering preview
* high-performance viewport drawing

Skia provides consistent rendering across macOS, Windows, and Linux.

### 12.9 Image Processing Libraries

Fleck uses Rust-native image processing where possible.

Responsibilities include:

* decoding images
* encoding images
* resizing
* cropping
* trimming
* compositing
* format conversion
* color operations
* alpha handling
* compression
* metadata handling

The image processing layer should be portable and should not depend on external binaries as a required part of the core app.

### 12.10 Background Removal Engine

Background removal is implemented as a local module.

It may use:

* ONNX Runtime
* local segmentation models
* Rust-side preprocessing
* Rust-side postprocessing

It should be:

* local-first
* optional if model size is large
* privacy-preserving
* integrated with layers and masks
* cancellable
* non-blocking

### 12.11 CLI

The Fleck CLI uses the same Rust core as the desktop app.

It supports:

* exporting `.fleck` workspaces
* exporting individual areas
* exporting all areas
* validating workspaces
* listing export areas
* listing outputs
* running recipes
* automating asset generation in repositories

The CLI exists so Fleck workspaces can become part of developer workflows and build pipelines.

## 13. Data Ownership

The ownership model is:

* Rust owns document state.
* Skia owns rendering.
* React owns UI.
* TanStack Query owns async cache state.
* Zustand owns local interaction state.
* Tauri owns native desktop integration.

This prevents the app from becoming a fragile web canvas editor and keeps the core reusable for desktop and CLI.

## 14. Performance and Responsiveness

Fleck should feel fast during common editing work.

Performance requirements:

* smooth pan and zoom
* low-latency brush strokes
* responsive selection movement
* fast layer toggling
* fast export preview generation
* background jobs do not block the UI
* large exports show progress
* undo/redo is responsive
* canvas interaction remains responsive during async operations where possible

Long operations include:

* background removal
* large image resize
* export all
* heavy compression
* large workspace loading
* large format conversion

These operations should be cancellable where possible.

## 15. Color, Transparency, and Quality

Fleck supports practical color handling.

Features:

* RGB editing
* alpha channel editing
* transparent background
* checkerboard preview
* color profiles where practical
* export profile handling
* background flattening
* transparency preservation
* high-quality scaling
* pixel-perfect scaling option
* nearest-neighbor scaling option
* smooth scaling option

Scaling modes:

* nearest neighbor
* bilinear
* bicubic
* Lanczos or equivalent high-quality mode

## 16. Clipboard and Drag-and-Drop

Fleck supports:

* paste image from clipboard
* copy selection to clipboard
* copy export result to clipboard
* drag image into workspace
* drag image out as exported file where supported
* drag export area output into Finder/Explorer where supported
* copy as Base64
* copy as data URI
* copy as Markdown image
* copy as CSS background image

Clipboard behavior should be useful for developer workflows.

## 17. Settings

Fleck settings include:

* theme
* accent color
* default export format
* default export location
* default padding
* default background
* default scaling mode
* transparency grid style
* keyboard shortcuts
* command palette behavior
* recent files
* autosave
* file format preferences
* background removal settings
* performance settings
* telemetry setting if telemetry exists

Telemetry, if included, must be opt-in.

## 18. Local-First and Privacy

Fleck is local-first.

Core expectations:

* no account required
* no cloud upload required
* image editing happens locally
* background removal happens locally where possible
* workspaces are stored locally
* exports are local files
* telemetry is absent or opt-in
* no vendor lock-in

This is important for developer trust.

## 19. Accessibility

Fleck should support:

* keyboard navigation
* accessible menus
* accessible dialogs
* readable contrast
* scalable UI text
* screen-reader labels for UI controls where practical
* reduced motion setting
* customizable shortcuts

The canvas itself is inherently visual, but surrounding controls should be accessible.

## 20. Cross-Platform Behavior

Fleck supports:

* macOS
* Windows
* Linux

Platform integration includes:

* native file dialogs
* native menus where appropriate
* platform keyboard shortcut conventions
* drag-and-drop
* clipboard
* open-with behavior
* recent files
* file associations
* system theme detection
* high-DPI display support

Fleck should feel consistent across platforms while respecting basic OS conventions.

## 21. Package Distribution

Fleck is distributed through common platform channels.

Distribution targets include:

* direct download
* Homebrew cask
* GitHub releases
* Windows installer
* winget
* Linux AppImage
* Flatpak
* package manager support where practical

The app name should work naturally in package-manager contexts.

Example:

```bash
brew install --cask fleck
```

## 22. Open Source Model

Fleck is open source.

The project should encourage contributions around:

* export presets
* recipes
* image formats
* platform packaging
* translations
* UI improvements
* bug fixes
* plugin commands
* documentation
* background removal improvements

The open-source identity should emphasize:

* local-first editing
* practical developer workflows
* transparent file format
* scriptable exports
* no account requirement
* no cloud dependency

## 23. Extension and Plugin System

Fleck supports a command-oriented extension model.

Extensions can add:

* commands
* export recipes
* export formats
* transform actions
* asset generators
* custom presets
* workspace validators

Plugin commands appear in the command palette.

Plugin design should preserve safety:

* clear permissions
* local execution model
* no silent network access
* explicit user approval for filesystem actions
* stable command API

## 24. Non-Goals

Fleck is not:

* a Photoshop replacement
* a Figma replacement
* a Canva replacement
* a digital painting suite
* a RAW photo editor
* a video editor
* a browser-only editor
* a cloud collaboration tool
* an AI image generator
* a full vector design tool

Fleck may include vector-like objects such as text and shapes, but its core identity is raster editing and asset export.

## 25. Defining Differentiator

Fleck’s main differentiator is not simply being open source, cross-platform, or command-palette-first.

Its defining feature is:

Raster editing with first-class export areas on an infinite workspace.

This means users can keep multiple assets in one workspace, edit them visually, define exactly what should be exported, and automate those outputs.

The product thesis is:

Traditional raster editors are document-first. Fleck is asset-workspace-first.
