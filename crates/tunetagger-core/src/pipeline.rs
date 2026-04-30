//! Pipeline orchestration will live here once the scan, recognition, metadata,
//! and tagging crates have concrete implementations.

use crate::model::ProcessingAudit;

#[derive(Debug, Default)]
pub struct PipelineOptions {
    pub dry_run: bool,
    pub write: bool,
    pub recursive: bool,
}

#[derive(Debug, Default)]
pub struct PipelineReport {
    pub audits: Vec<ProcessingAudit>,
}
