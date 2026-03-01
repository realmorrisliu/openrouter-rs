use serde::{Deserialize, Serialize};

/// Common pagination input used by paginated OpenRouter endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PaginationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl PaginationOptions {
    pub fn new(offset: Option<u32>, limit: Option<u32>) -> Self {
        Self { offset, limit }
    }

    pub fn with_offset(offset: u32) -> Self {
        Self {
            offset: Some(offset),
            limit: None,
        }
    }

    pub fn with_limit(limit: u32) -> Self {
        Self {
            offset: None,
            limit: Some(limit),
        }
    }

    pub fn with_offset_and_limit(offset: u32, limit: u32) -> Self {
        Self {
            offset: Some(offset),
            limit: Some(limit),
        }
    }

    pub fn to_query_pairs(self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(offset) = self.offset {
            pairs.push(("offset", offset.to_string()));
        }

        if let Some(limit) = self.limit {
            pairs.push(("limit", limit.to_string()));
        }

        pairs
    }
}
