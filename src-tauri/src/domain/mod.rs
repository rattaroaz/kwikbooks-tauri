//! Domain services: journals, posting, lifecycle, reporting (Phase 4).

pub mod accounts;
pub mod constants;
pub mod dates;
pub mod import;
pub mod ids;
pub mod journal;
pub mod lifecycle;
pub mod lists;
pub mod money;
pub mod posting;
pub mod reports;
pub mod search;

#[cfg(test)]
mod invariants;
