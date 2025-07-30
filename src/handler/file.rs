// Uploading, processing, and encryption before saving to db

use std::{fs, path::PathBuf, sync::Arc};

use axum::{body::Body, extract::Multipart, http::{Response, StatusCode}, response::IntoResponse, routing::post, Extension, Json, Router};
use chrono::{DateTime, Utc};
use rsa::{pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey}, RsaPrivateKey, RsaPublicKey};
use validator::Validate;
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{db::UserExt, dtos::{FileUploadDtos, Response as ResponseDto, RetrieveFileDto}, error::HttpError, middleware::JWTAuthMiddleware, utils::{decrypt::decrypt_file, encrypt::encrypt_file, password}, AppState};

//Upload
pub async fn upload_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart
) -> Result<impl IntoResponse, HttpError> {

    let mut file_dataa = Vec::new();
    let mut file_name = String::new();
    let mut file_size: i64 = 0;
    let mut form_data = FileUploadDtos {
        recipient_email: String::new(),
        password: String::new(),
        expiration_date: String::new(),
    };

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "fileUpload" => {
                file_name = field.file_name().unwrap_or("Unknown_file").to_string();
                file_data = field.bytes().await.unwrap().to_vec();
                file_size = file_data.len() as i64;
            },
            "recipient_email" => {
                .form_data.recipient_email = field.text().await.unwrap();
            },
            "password" => {
                form_data.password = field.text().await.unwrap();
            },
            "expiration_date" => {
                form_data.expiration_date = field.text().await.unwrap();
            },
            _ => {}
        }
    }

    form_data.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let recipient_result = app_state.db_client
        .get_user(None, None, Some(&form_data.recipient_email))
        .await
        .map_err(|e| HttpError::server_error(e.to_string))?;

    let recipient_user = recipient_result.ok_or(HttpError::bad_request("Recipient user not found"))?;

    let public_key_str = match &recipient_user.public_key {
        Some(key) => key,
        None => return Err(HttpError::bad_request("User has no public key")),
    };

    //let public_key_bytes//

}