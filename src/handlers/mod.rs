use axum::{extract::{Extension, Path}, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::PgPool;
use rand::{Rng, thread_rng};

use crate::models::Ref;

const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn generate_random_base32_code() -> String {
    let mut rng = thread_rng();
    (0..8).map(|_| {
        let idx = rng.gen_range(0..BASE32_ALPHABET.len());
        BASE32_ALPHABET[idx] as char
    }).collect()
}

pub async fn get_ref(
    Extension(pool): Extension<PgPool>,
    Path(code): Path<String>,
) -> Result<Json<Ref>, (StatusCode, String)> {
    match Ref::select_by_code(&pool, &code).await {
        Ok(Some(r)) => Ok(Json(r)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Ref not found".to_string())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateRefRequest {
    pub referrer_code: Option<String>
}

pub async fn create_ref(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateRefRequest>,
) -> Result<(StatusCode, Json<Ref>), (StatusCode, String)> {
    let referrer_code = payload.referrer_code.as_deref().unwrap_or("").trim();
    let new_code = generate_random_base32_code();

    let new_path = if referrer_code.is_empty() {
        // Root ref
        new_code.clone()
    } else {
        // Child ref
        let parent = match Ref::select_by_code(&pool, referrer_code).await {
            Ok(Some(p)) => p,
            Ok(None) => return Err((StatusCode::BAD_REQUEST, "Invalid referrer code".to_string())),
            Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        };
        format!("{}.{}", parent.path, new_code)
    };

    match Ref::insert(&pool, &new_code, &new_path).await {
        Ok(r) => {
            // Increment points for ancestors up to 5 steps
            if !referrer_code.is_empty() {
                let path_parts: Vec<&str> = new_path.split('.').collect();
                let mut current_points = 5;
                for i in (0..path_parts.len() - 1).rev().take(5) {
                    let ancestor_code = path_parts[i];
                    if let Err(err) = Ref::update_point(&pool, ancestor_code, current_points).await {
                        eprintln!("Failed to update points for ancestor {}: {}", ancestor_code, err);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update ancestor points".into()));
                    }
                    current_points -= 1;
                    if current_points == 0 {
                        break;
                    }
                }
            }
            Ok((StatusCode::CREATED, Json(r)))
        }
        Err(err) => {
            eprintln!("failed to insert ref: {err}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to insert ref".into()))
        }
    }
}
