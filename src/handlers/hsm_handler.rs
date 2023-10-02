use crate::utils::sign_transaction::{sign_erc20, sign_raw_tx, TxBroadcastRequest};
use axum::{http::StatusCode, response::IntoResponse, Json};
// use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// async fn login_handler(body: Json<LoginRequest>) -> Json<String> {
//     // Assuming these are the valid credentials for demonstration purposes
//     let valid_username = dotenvy::var("USERNAME").expect("Username must set");
//     let valid_password = dotenvy::var("PASSWORD").expect("Username must set");

//     // Validate the credentials
//     if body.username == valid_username && body.password == valid_password {
//         // Credentials are valid
//         let user_id = "12345"; // Fetch user ID from your authentication logic

//         // Generate a JWT token
//         let claims = Claims {
//             sub: user_id.to_string(),
//             role: "admin".to_string(), // Adjust as needed based on your application's logic
//         };
//         let secret_key = dotenvy::var("SECRET_KEY").expect("Secret key cannot be empty");
//         let token = encode(
//             &Header::default(),
//             &claims,
//             &EncodingKey::from_secret(secret_key.as_ref()), // Replace with your secret key
//         )
//         .unwrap();

//         Json(token)
//     } else {
//         // Invalid credentials
//         Json("Invalid credentials".to_string())
//     }
// }

pub async fn sign_erc20_transaction_handler(
    Json(payload): Json<TxBroadcastRequest>,
    // user: axum::extract::Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user has the necessary role to sign transactions
    // if user.role != "admin" {
    //     let error_message = "Unauthorized user";
    //             let json_response = serde_json::json!({
    //                 "status": "fail",
    //                 "data": error_message
    //             });
    //     return Ok(Json(json_response));
    // }
    // Process the transaction signing using the HSM
    // Replace this with your actual transaction signing logic
    println!(" ========= Payload: {:#?}", &payload);
    let signed_transaction = match sign_erc20(&payload).await {
        Ok(tx) => tx,
        Err(err) => {
            let error_message = format!("Error retrieving origin network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": signed_transaction
    });
    Ok(Json(json_response))
}

pub async fn sign_raw_transaction_handler(
    Json(payload): Json<TxBroadcastRequest>,
    // user: axum::extract::Extension<Claims>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user has the necessary role to sign transactions
    // if user.role != "admin" {
    //     let error_message = "Unauthorized user";
    //             let json_response = serde_json::json!({
    //                 "status": "fail",
    //                 "data": error_message
    //             });
    //     return Ok(Json(json_response));
    // }
    // Process the transaction signing using the HSM
    // Replace this with your actual transaction signing logic
    println!(" ========= Payload: {:#?}", &payload);
    let signed_transaction = match sign_raw_tx(&payload).await {
        Ok(tx) => tx,
        Err(err) => {
            let error_message = format!("Error retrieving origin network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": signed_transaction
    });
    Ok(Json(json_response))
}
