use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    50
}

impl PaginationParams {
    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("page must be >= 1".to_string());
        }
        if self.page_size < 1 || self.page_size > 100 {
            return Err("page_size must be between 1 and 100".to_string());
        }
        Ok(())
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u32,
    pub page_size: u32,
    pub total_items: i64,
    pub total_pages: u32,
}

impl PaginationMeta {
    pub fn new(page: u32, page_size: u32, total_items: i64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
        Self {
            page,
            page_size,
            total_items,
            total_pages,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u32, page_size: u32, total_items: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(page, page_size, total_items),
        }
    }
}
