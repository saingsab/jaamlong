use axum::{
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    role: String,
    iat: usize,
    exp: usize,
}

pub async fn auth<B>(
    cookie_jar: CookieJar,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Retrieve token from cookie or Authorization header
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    auth_value
                        .strip_prefix("Bearer ")
                        .map(|bearer| bearer.to_owned())
                })
        });

    // Ensure a valid token is present
    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;
    // Decode token and extract claims
    let secret_key = dotenvy::var("SECRET_KEY").expect("Secret key cannot be empty");
    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        println!("Error: {}", e);
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?
    .claims;

    // Parse user IDs and validate
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid parse token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;
    let user_id_db = uuid::Uuid::parse_str(&dotenvy::var("USER_ID").expect("User ID must be set"))
        .map_err(|_| {
            let json_error = ErrorResponse {
                status: "fail",
                message: "Invalid parse token".to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(json_error))
        })?;

    if claims.role != "admin" {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid not equal token".to_string(),
        };
        return Ok(Err((StatusCode::UNAUTHORIZED, Json(json_error))));
    }

    //Check user ID from token against stored user ID
    if user_id.ne(&user_id_db) {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid not equal token".to_string(),
        };
        Ok(Err((StatusCode::UNAUTHORIZED, Json(json_error))))
    } else {
        // Retrieve username from environment
        let user = dotenvy::var("USERNAME").map_err(|e| {
            let json_error = ErrorResponse {
                status: "fail",
                message: format!("Error fetching user from database: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

        // Insert username into request extensions
        req.extensions_mut().insert(user);
        Ok(Ok(next.run(req).await))
    }
}

pub async fn login_handler(
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Assuming these are the valid credentials for demonstration purposes
    let valid_username = dotenvy::var("USERNAME").expect("Username must set");
    let valid_password = dotenvy::var("PASSWORD").expect("Username must set");

    // Validate the credentials
    if body.username == valid_username && body.password == valid_password {
        // Credentials are valid
        let user_id = dotenvy::var("USER_ID").expect("User ID must set"); // Fetch user ID from your authentication logic

        let now = chrono::Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + chrono::Duration::minutes(99999999)).timestamp() as usize;
        // Generate a JWT token
        let claims = Claims {
            sub: user_id,
            role: "admin".to_string(),
            iat,
            exp,
        };
        let secret_key = dotenvy::var("SECRET_KEY").expect("Secret key cannot be empty");
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
        .unwrap();

        Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "data": "success",
                "token": token
            })),
        ))
    } else {
        // Invalid credentials
        Ok((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!("Error: Invalid credentials".to_string())),
        ))
    }
}
