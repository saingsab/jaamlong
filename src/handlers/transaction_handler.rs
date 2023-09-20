use crate::AppState;
use crate::{
    models::{
        network::{Network, ResponseNetwork},
        token_address::TokenAddress,
        transaction::{RequestInsertTx, Transaction},
        transaction_status::{RequestInsertTxStatus, TransactionStatus},
    },
    utils::transaction_module::{
        generate_error_response, get_base_fee, get_confirmed_block, get_est_gas_price, get_tx,
        get_tx_receipt, send_erc20_token, send_raw_transaction, token_converter,
        validate_account_balance,
    },
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;
use web3::{
    ethabi::{decode, ParamType},
    types::{Address, BlockId, CallRequest, U256, U64},
};

#[derive(Deserialize)]
pub struct RequestedTransaction {
    sender_address: String,
    receiver_address: String,
    origin_network: Option<Uuid>,
    destin_network: Option<Uuid>,
    from_token_address: Uuid,
    to_token_address: Uuid,
    transfer_amount: f64,
    created_by: Option<Uuid>,
}

#[derive(Serialize, Debug)]
pub struct ResponseTransaction {
    id: Uuid,
    sender_address: String,
    receiver_address: String,
    transfer_amount: String,
    gas_limit: String,
    max_priority_fee_per_gas: i64,
    max_fee_per_gas: u128,
}

#[derive(Deserialize)]
pub struct TransactionHash {
    id: Uuid,
    hash: String,
}

pub async fn request_tx(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<RequestedTransaction>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //validate minimun transfer amount
    if payload.transfer_amount <= 0.00 {
        return Ok(Json(
            generate_error_response("Amount must be greater than zero").unwrap(),
        ));
    } else if payload.origin_network == payload.destin_network {
        return Ok(Json(
            generate_error_response("Same Network Not Allowed").unwrap(),
        ));
    }
    //validate origin network
    let validated_origin_network: ResponseNetwork;
    if let Some(origin_network) = payload.origin_network {
        match Network::get_network_by_id(&data.db, origin_network).await {
            Ok(network) => {
                validated_origin_network = network;
            }
            Err(err) => {
                let error_message = format!("Error retrieving origin network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        }
    } else {
        let json_response = serde_json::json!({
            "status": "fail",
            "data": "Origin Network Not Found"
        });
        return Ok(Json(json_response));
    }
    //validate destination network
    let validated_destinated_network: ResponseNetwork;
    if let Some(destinated_network) = payload.destin_network {
        match Network::get_network_by_id(&data.db, destinated_network).await {
            Ok(network) => {
                validated_destinated_network = network;
            }
            Err(err) => {
                let error_message = format!("Error retrieving destin network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    } else {
        let json_response = serde_json::json!({
            "status": "fail",
            "data": "Destination Network Not Found"
        });
        return Ok(Json(json_response));
    }
    //validate tokenID
    let validated_from_token =
        match TokenAddress::get_token_address_by_id(&data.db, payload.from_token_address).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error Token Address: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    let validated_to_token =
        match TokenAddress::get_token_address_by_id(&data.db, payload.to_token_address).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error Token Address: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    // validate asset type
    if validated_from_token.asset_type != "0"
        && validated_from_token.asset_type != "1"
        && validated_to_token.asset_type != "0"
        && validated_to_token.asset_type != "1"
    {
        let error_message = "Asset type not supported";
        let json_response = serde_json::json!({
            "status": "fail",
            "data": error_message
        });
        return Ok(Json(json_response));
    }
    // perform token conversion
    let transfer_value = match token_converter(
        &data.db,
        validated_origin_network.id,
        validated_from_token.asset_type.clone(),
        Some(validated_from_token.id),
        payload.transfer_amount,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error converting bridge fee to token value: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    // Calculation of the bridge fee
    let base_bridge_fee = validated_destinated_network.bridge_fee;
    let bridge_fee = base_bridge_fee * payload.transfer_amount;
    let bridge_fee_value = match token_converter(
        &data.db,
        validated_origin_network.id,
        validated_from_token.asset_type.clone(),
        Some(validated_from_token.id),
        bridge_fee,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error converting bridge fee token value: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    // validate sender account
    let sender_address = match Address::from_str(payload.sender_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            let error_message = format!("Error parsing address: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    if validated_from_token.asset_type == "0" {
        match validate_account_balance(
            &data.db,
            validated_origin_network.id,
            sender_address,
            transfer_value,
            None,
            bridge_fee_value,
        )
        .await
        {
            Ok(_) => {
                println!("Account Validation Passed");
            }
            Err(err) => {
                let error_message = format!("Account Balance Insufficient: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        }
    } else if validated_from_token.asset_type == "1" {
        match validate_account_balance(
            &data.db,
            validated_origin_network.id,
            sender_address,
            transfer_value,
            Some(validated_from_token.id),
            bridge_fee_value,
        )
        .await
        {
            Ok(_) => {
                println!("Account Validation Passed");
            }
            Err(err) => {
                let error_message = format!("Account Balance Insufficient: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        }
    }
    //estimated_gas_price and balance validation
    let est_gas_price = match get_est_gas_price(
        &data.db,
        validated_origin_network.id,
        CallRequest::builder().build(),
    )
    .await
    {
        Ok(gas_price) => gas_price,
        Err(err) => {
            let error_message = format!("Error retrieving est gas price: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    let tx_status = RequestInsertTxStatus {
        status_name: String::from("Pending"),
        created_by: Some(Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            payload.sender_address.clone().as_bytes(),
        )),
    };
    //create new transaction status
    let transaction_status_id = match TransactionStatus::create(&data.db, tx_status).await {
        Ok(tx) => tx.id,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let transfer_amount_big_dec = match BigDecimal::from_str(transfer_value.to_string().as_str()) {
        Ok(value) => value,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Fail to parse value to big decimal: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let bridge_value_big_dec = match BigDecimal::from_str(bridge_fee_value.to_string().as_str()) {
        Ok(value) => value,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Fail to parse value to big decimal: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let inserted_tx = RequestInsertTx {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        from_token_address: validated_from_token.id.to_string(),
        to_token_address: validated_to_token.id.to_string(),
        origin_network: Some(validated_origin_network.id),
        destin_network: Some(validated_destinated_network.id),
        transfer_amount: transfer_amount_big_dec,
        bridge_fee: bridge_value_big_dec,
        tx_status: Some(transaction_status_id),
        origin_tx_hash: None,
        destin_tx_hash: None,
        created_by: payload.created_by,
    };
    //insert unconfirmed tx to database
    let created_tx = match Transaction::create(&data.db, inserted_tx).await {
        Ok(tx) => tx,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Failed to insert transaction{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let base_fee = match get_base_fee(&data.db, validated_origin_network.id).await {
        Ok(block) => match block.base_fee_per_gas {
            Some(base_gas_fee) => base_gas_fee,
            None => U256::from(0),
        },
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Fail to get based fee: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let bridge_address = validated_destinated_network.bridge_address;
    let total_transfer_amount =
        match U256::from(bridge_fee_value).checked_add(U256::from(transfer_value)) {
            Some(amount) => amount,
            None => {
                let json_response = serde_json::json!({
                    "status": "success",
                    "data": "Error calulating transfer amount"
                });
                return Ok(Json(json_response));
            }
        };
    let response_tx = ResponseTransaction {
        id: created_tx.id,
        sender_address: payload.sender_address.clone(),
        receiver_address: bridge_address,
        transfer_amount: total_transfer_amount.to_string(),
        gas_limit: est_gas_price.to_string(),
        max_priority_fee_per_gas: 0,
        max_fee_per_gas: base_fee.as_u128(),
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": response_tx
    });
    Ok(Json(json_response))
}

pub async fn broadcast_tx(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<TransactionHash>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //validate transaction hash
    if &payload.hash[0..2] != "0x" {
        let json_response = serde_json::json!({
            "status": "fail",
            "data": format!("Transaction hash is incorrect")
        });
        return Ok(Json(json_response));
    }
    // query transaction from db
    let transaction = match Transaction::get_transaction(&data.db, payload.id).await {
        Ok(tx) => tx,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    //get bridge address
    let destination_network = match transaction.destin_network {
        Some(network) => network,
        None => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": "Destination Network Not Found"
            });
            return Ok(Json(json_response));
        }
    };
    // query network id from transaction id
    let network = match Network::get_network_by_id(&data.db, destination_network).await {
        Ok(network_id) => network_id,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let bridge_address = match Address::from_str(network.bridge_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err parsing bridge address: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let to_token = match TokenAddress::get_token_address_by_id(
        &data.db,
        Uuid::from_str(transaction.to_token_address.as_str())
            .expect("Failed to parse to_token_address"),
    )
    .await
    {
        Ok(token) => token,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let from_token = match TokenAddress::get_token_address_by_id(
        &data.db,
        Uuid::from_str(transaction.from_token_address.as_str())
            .expect("Fail to parse to_token_address"),
    )
    .await
    {
        Ok(token) => token,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    //validate receive transaction
    if from_token.asset_type == "0" {
        // for native token type, get transaciton from hash
        match get_tx(
            &data.db,
            transaction.origin_network.unwrap(),
            payload.hash.clone(),
        )
        .await
        {
            Ok(tx) => {
                //validate from address
                match Some(tx.from) {
                    Some(from_address) => {
                        if from_address
                            != Some(Address::from_str(&transaction.sender_address).unwrap())
                        {
                            let json_response = serde_json::json!({
                                "status": "fail",
                                "data": format!("From address does not match")
                            });
                            return Ok(Json(json_response));
                        }
                    }
                    None => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("From address not found")
                        });
                        return Ok(Json(json_response));
                    }
                }
                //validate to_address
                match Some(tx.to) {
                    Some(to_address) => {
                        if to_address != Some(bridge_address) {
                            let json_response = serde_json::json!({
                                "status": "fail",
                                "data": format!("From address does not match")
                            });
                            return Ok(Json(json_response));
                        }
                    }
                    None => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("From address not found")
                        });
                        return Ok(Json(json_response));
                    }
                }
                // validate the value transfer
                let transfer_value_int = match BigDecimal::from_str(tx.value.to_string().as_str()) {
                    Ok(value) => value,
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Error parsing transfer value: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                };
                println!("Transfer value: {:#?}", &transfer_value_int);
                println!("Bridge fee: {}", &transaction.bridge_fee);
                let actual_transfer_amount = transfer_value_int - transaction.bridge_fee.clone();
                if actual_transfer_amount != transaction.transfer_amount.clone() {
                    let json_response = serde_json::json!({
                        "status": "fail",
                        "data": format!("Value does not match")
                    });
                    return Ok(Json(json_response));
                }
                // validate confirmation block
                match get_confirmed_block(
                    &data.db,
                    transaction.origin_network.unwrap(),
                    BlockId::Hash(tx.block_hash.unwrap()),
                )
                .await
                {
                    Ok(block_confirmation) => {
                        //check if block_confirmation is greater than 2. Negative numbers return None
                        match &block_confirmation.checked_sub(U64::from(2)) {
                            Some(_block_num) => {
                                println!(
                                    "Success, Number of Confirmation Blocks: {}",
                                    &block_confirmation
                                );
                            }
                            None => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Block confirmation less than 2")
                                });
                                return Ok(Json(json_response));
                            }
                        }
                        println!("Block Confirmation: {:#?}", block_confirmation);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                }
            }
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                return Ok(Json(json_response));
            }
        };
    } else if from_token.asset_type == "1" {
        // for erc20 asset type
        match get_tx_receipt(
            &data.db,
            transaction.origin_network.unwrap(),
            payload.hash.clone(),
        )
        .await
        {
            Ok(tx) => {
                let logs = tx.logs.into_iter();
                let mut topics = Vec::new();
                if let Some(log) = logs.into_iter().next() {
                    for topic in log.topics.into_iter() {
                        topics.push(topic);
                    }
                    let bytes_data = log.data;
                    // Convert web3::types::Bytes to &[u8]
                    let byte_slice = &bytes_data.0;
                    // Interpret the byte slice based on the specific data format (ABI)
                    let abi_type = ParamType::Uint(256);
                    // Decode the byte slice
                    let decoded_data = decode(&[abi_type], byte_slice).expect("Decoding failed");
                    // Extract and display the decoded value
                    let value = &decoded_data[0];
                    println!("Value from log: {:#?}", &value);
                    let transfer_value_int = match value.clone().into_uint() {
                        Some(value) => BigDecimal::from_str(value.to_string().as_str()).unwrap(),
                        None => {
                            let json_response = serde_json::json!({
                            "status": "fail",
                            "data": "Error converting transfer value to int"
                             });
                            return Ok(Json(json_response));
                        }
                    };
                    println!("Transaction: {:#?}", transaction);
                    let bridge_fee_parse = match BigDecimal::from_str(<&str>::clone(
                        &transaction.bridge_fee.to_string().as_str(),
                    )) {
                        Ok(value) => value,
                        Err(err) => {
                            let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Error parsing bridge fee: {}", err)
                                });
                            return Ok(Json(json_response));
                        }
                    };
                    let actual_transfer_amount = transfer_value_int - bridge_fee_parse;
                    // validate the value transfer
                    if actual_transfer_amount != transaction.transfer_amount.clone() {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Value does not match")
                        });
                        return Ok(Json(json_response));
                    }
                }
                let from_address = Address::from(topics[1]); //sender address
                let to_address = Address::from(topics[2]); //receiver address
                if from_address //validate from address
                    != Address::from_str(&transaction.sender_address)
                        .expect("Error decoding address")
                {
                    let json_response = serde_json::json!({
                        "status": "fail",
                        "data": format!("To address does not match")
                    });
                    return Ok(Json(json_response));
                }
                // validate to address which must be bridge address
                if to_address != bridge_address {
                    let json_response = serde_json::json!({
                        "status": "fail",
                        "data": format!("To address does not match")
                    });
                    return Ok(Json(json_response));
                }
                // validate confirmation block
                match get_confirmed_block(
                    &data.db,
                    transaction.origin_network.unwrap(),
                    BlockId::Hash(tx.block_hash.unwrap()),
                )
                .await
                {
                    Ok(block_confirmation) => {
                        //check if block_confirmation is greater than 2. Negative numbers return None
                        match &block_confirmation.checked_sub(U64::from(2)) {
                            Some(_block_num) => {
                                println!(
                                    "Success, Number of Confirmation Blocks: {}",
                                    &block_confirmation
                                );
                            }
                            None => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Block confirmation less than 2")
                                });
                                return Ok(Json(json_response));
                            }
                        }
                        println!("Block Confirmation: {:#?}", block_confirmation);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                }
            }
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err Transaction Receipt: {}", err)
                });
                return Ok(Json(json_response));
            }
        }
    }
    // validate transaction status from db
    match TransactionStatus::get_transaction_status(&data.db, transaction.tx_status.unwrap()).await
    {
        Ok(tx_status) => {
            if tx_status.status_name != "Pending" {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Transaction not in Pending state. Current State: {}", tx_status.status_name)
                });
                return Ok(Json(json_response));
            }
            println!(
                "Transaciton ID: {}, \nStatus: {:#?}",
                &payload.id, tx_status
            );
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    }
    //update the origin transaciton hash
    let origin_tx_hash = Some(payload.hash.clone());
    match Transaction::update_tx_hash(&data.db, payload.id, origin_tx_hash, None).await {
        Ok(tx) => {
            println!("Transaction after updating origin hash: {:#?}", tx);
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    }
    // //validate token amount in the pool
    // match get_token_supply(&data.db, network.id, transaction.to_asset_type.unwrap()).await {
    //     Ok(total_supply) => {
    //         if total_supply.is_zero() {
    //             let json_response = serde_json::json!({
    //                 "status": "fail",
    //                 "data": format!("Total Supply is zero")
    //             });
    //             return Ok(Json(json_response));
    //         } else {
    //             total_supply
    //         }
    //     }
    //     Err(err) => {
    //         let json_response = serde_json::json!({
    //             "status": "fail",
    //             "data": format!("Err: {}", err)
    //         });
    //         return Ok(Json(json_response));
    //     }
    // };
    let p_k: String = dotenvy::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    if to_token.asset_type == "0" {
        match send_raw_transaction(&data.db, network.id, &transaction, p_k.as_str()).await {
            Ok(tx) => {
                std::thread::sleep(std::time::Duration::from_secs(5));
                // get transaciton receipt from hash
                let new_tx_receipt = match get_tx(&data.db, network.id, format!("{:?}", tx)).await {
                    Ok(tx) => tx,
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Error getting transaction: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                };
                //insert destination hash to db
                match Transaction::update_tx_hash(
                    &data.db,
                    transaction.id,
                    Some(payload.hash),
                    Some(format!("{:?}", tx)),
                )
                .await
                {
                    Ok(updated_tx) => {
                        println!("Update tx: {:?}", updated_tx);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err updating tx: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                }
                println!("New Tx: {:#?}", &new_tx_receipt);
                //check block confirmation
                std::thread::sleep(std::time::Duration::from_secs(5));
                match get_confirmed_block(
                    &data.db,
                    network.id,
                    BlockId::Hash(new_tx_receipt.block_hash.expect("Failed to get block hash")),
                )
                .await
                {
                    Ok(num_block_confirmation) => {
                        println!("Block confirmation #######: {}", num_block_confirmation);
                        if num_block_confirmation.checked_sub(U64::from(2)).is_none() {
                            std::thread::sleep(std::time::Duration::from_secs(5));
                        }
                        //check if block_confirmation is greater than 2. Negative numbers return None
                        match &num_block_confirmation.checked_sub(U64::from(2)) {
                            Some(_block_num) => {
                                println!(
                                    "Success, Number of Confirmation Blocks: {}",
                                    &num_block_confirmation,
                                );
                            }
                            None => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Block confirmation less than 2")
                                });
                                return Ok(Json(json_response));
                            }
                        }
                        println!("Block confirmation: {:#?}", num_block_confirmation);
                        match get_tx_receipt(
                            &data.db,
                            transaction.destin_network.unwrap(),
                            format!("{:?}", tx),
                        )
                        .await
                        {
                            Ok(tx) => {
                                match tx.status {
                                    Some(status) => {
                                        if status == U64::from(1) {
                                            //update tx status to success
                                            match TransactionStatus::update_status(
                                                &data.db,
                                                transaction.tx_status.unwrap(),
                                                "Success".to_string(),
                                            )
                                            .await
                                            {
                                                Ok(_status) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Transaction Status: {}\nBroadcast Tx Hash: {:#?}", tx.status.unwrap(), tx.transaction_hash)
                                                    });
                                                    Ok(Json(json_response))
                                                }
                                                Err(err) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Err Updating Tx Status: {}", err)
                                                    });
                                                    Ok(Json(json_response))
                                                }
                                            }
                                        } else if status == U64::from(0) {
                                            //update tx status to fail
                                            match TransactionStatus::update_status(
                                                &data.db,
                                                transaction.tx_status.unwrap(),
                                                "Fail".to_string(),
                                            )
                                            .await
                                            {
                                                Ok(_status) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Transaction Status: {}, Broadcast Tx Hash: {:#?}", tx.status.unwrap(), tx.transaction_hash)
                                                    });
                                                    return Ok(Json(json_response));
                                                }
                                                Err(err) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Err Updating Tx Status: {}", err)
                                                    });
                                                    return Ok(Json(json_response));
                                                }
                                            }
                                        } else {
                                            let json_response = serde_json::json!({
                                                "status": "fail",
                                                "data": format!("Status not found")
                                            });
                                            return Ok(Json(json_response));
                                        }
                                    }
                                    None => {
                                        let json_response = serde_json::json!({
                                            "status": "fail",
                                            "data": format!("Status not found")
                                        });
                                        Ok(Json(json_response))
                                    }
                                }
                            }
                            Err(err) => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Err Updating Tx Status: {}", err)
                                });
                                Ok(Json(json_response))
                            }
                        }
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err Getting Block Confirmation: {}", err)
                        });
                        Ok(Json(json_response))
                    }
                }
            }
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                Ok(Json(json_response))
            }
        }
    } else if to_token.asset_type == "1" {
        match send_erc20_token(&data.db, network.id, &transaction, p_k.as_str()).await {
            Ok(tx) => {
                let new_tx_receipt = match get_tx(&data.db, network.id, format!("{:?}", tx)).await {
                    Ok(tx) => tx,
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err Query Tx Hash: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                };
                //insert destination hash to db
                match Transaction::update_tx_hash(
                    &data.db,
                    transaction.id,
                    Some(payload.hash),
                    Some(format!("{:?}", tx)),
                )
                .await
                {
                    Ok(updated_tx) => {
                        println!("Update tx: {:?}", updated_tx);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err Updating Tx: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                }
                //check block confirmation
                match get_confirmed_block(
                    &data.db,
                    network.id,
                    BlockId::Hash(new_tx_receipt.block_hash.unwrap()),
                )
                .await
                {
                    Ok(num_block_confirmation) => {
                        //check if block_confirmation is greater than 2. Negative numbers return None
                        match &num_block_confirmation.checked_sub(U64::from(2)) {
                            Some(_block_num) => {
                                println!(
                                    "Success, Number of Confirmation Blocks: {}",
                                    &num_block_confirmation
                                );
                            }
                            None => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Block confirmation less than 2")
                                });
                                return Ok(Json(json_response));
                            }
                        }
                        println!("Block confirmation: {:#?}", num_block_confirmation);
                        match get_tx_receipt(&data.db, network.id, format!("{:?}", tx)).await {
                            Ok(tx) => {
                                match tx.status {
                                    Some(status) => {
                                        println!("Status: {:?}", status);
                                        if status == U64::from(1) {
                                            //update tx status to success
                                            match TransactionStatus::update_status(
                                                &data.db,
                                                transaction.tx_status.unwrap(),
                                                "Success".to_string(),
                                            )
                                            .await
                                            {
                                                Ok(status) => {
                                                    println!("Tx Status: {:#?}", status);
                                                }
                                                Err(err) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Err: {}", err)
                                                    });
                                                    return Ok(Json(json_response));
                                                }
                                            }
                                        } else if status == U64::from(0) {
                                            //update tx status to fail
                                            match TransactionStatus::update_status(
                                                &data.db,
                                                transaction.tx_status.unwrap(),
                                                "Fail".to_string(),
                                            )
                                            .await
                                            {
                                                Ok(status) => {
                                                    println!("Tx Status: {:#?}", status);
                                                }
                                                Err(err) => {
                                                    let json_response = serde_json::json!({
                                                        "status": "fail",
                                                        "data": format!("Err: {}", err)
                                                    });
                                                    return Ok(Json(json_response));
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        let json_response = serde_json::json!({
                                            "status": "fail",
                                            "data": format!("Status not found")
                                        });
                                        return Ok(Json(json_response));
                                    }
                                }
                                println!("Broadcast transaction: {:#?}", &new_tx_receipt);
                                let json_response = serde_json::json!({
                                    "status": "success",
                                    "data": &new_tx_receipt
                                });
                                Ok(Json(json_response))
                            }
                            Err(err) => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Err Getting Transaction Receipt: {}", err)
                                });
                                return Ok(Json(json_response));
                            }
                        }
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                }
            }
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                Ok(Json(json_response))
            }
        }
    } else {
        let json_response = serde_json::json!({
            "status": "fail",
            "data": format!("Asset Type Not Supported")
        });
        Ok(Json(json_response))
    }
}
