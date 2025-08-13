use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub error_code: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
    pub total_pages: i32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
            error_code: None,
        }
    }

    pub fn error(message: &str, error_code: Option<&str>) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
            error_code: error_code.map(|s| s.to_string()),
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: i32, per_page: i32, total: i64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            success: true,
            data,
            pagination: PaginationInfo {
                page,
                per_page,
                total,
                total_pages,
                has_next,
                has_prev,
            },
            message: "Data retrieved successfully".to_string(),
        }
    }
}