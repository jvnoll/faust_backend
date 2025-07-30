use std::sync::Arc;

use axum::{extract::Query, response::IntoResponse, Extension};
use validator::Validate;

use crate::{dtos::RequestQueryDto, error::HttpError, middleware::JWTAuthMiddleware, AppState};



pub fn get_file_list_handler() -> Router {}

pub async fn get_user_shared_files(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    query_params.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;
    
}