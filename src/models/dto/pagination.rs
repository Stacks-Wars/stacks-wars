use serde::{Deserialize, Serialize};

/// Pagination helper for list queries
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub limit: i64,
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        (self.page.saturating_sub(1)) * self.limit
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, limit: 20 }
    }
}

/// Generic paginated response wrapper
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

/// Pagination metadata for responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page: u32,
    pub limit: u32,
    pub total_count: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}
