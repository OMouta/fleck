//! Core application engine for Fleck.
//!
//! This crate owns authoritative document state. React, Tauri, and rendering
//! layers must access document behavior through explicit core APIs.

pub const APP_NAME: &str = "Fleck";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnershipBoundary {
    pub owner: &'static str,
    pub responsibility: &'static str,
}

pub fn ownership_boundaries() -> &'static [OwnershipBoundary] {
    &[
        OwnershipBoundary {
            owner: "Rust core",
            responsibility: "document state and command execution",
        },
        OwnershipBoundary {
            owner: "Skia",
            responsibility: "viewport rendering",
        },
        OwnershipBoundary {
            owner: "React",
            responsibility: "interface and immediate UI state",
        },
        OwnershipBoundary {
            owner: "Tauri",
            responsibility: "native shell and secure bridge",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_declares_document_ownership() {
        assert!(
            ownership_boundaries()
                .iter()
                .any(|boundary| boundary.owner == "Rust core"
                    && boundary.responsibility.contains("document state"))
        );
    }
}
