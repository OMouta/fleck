//! Rendering integration boundary for Fleck.
//!
//! Skia-backed rendering will live behind this crate. It may render document
//! data, but it must not become the source of truth for document state.

pub fn renderer_boundary_summary() -> &'static str {
    "fleck-render renders core-owned document state"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_boundary_names_core_ownership() {
        assert!(renderer_boundary_summary().contains("core-owned document state"));
    }
}
