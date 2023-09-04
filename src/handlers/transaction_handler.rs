use crate::utils::transaction_module::get_decimal;
use crate::AppState;
use crate::{
    models::{
        bridge::Bridge,
        network::Network,
        token_address::TokenAddress,
        transaction::{RequestInsertTx, Transaction},
        transaction_status::{RequestInsertTxStatus, TransactionStatus},
    },
    utils::transaction_module::{
        get_base_fee, get_confirmed_block, get_est_gas_price, get_gas_price, get_token_supply,
        get_tx, get_tx_receipt, send_erc20_token, send_raw_transaction, token_converter,
        validate_account,
    },
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use web3::ethabi::{decode, ParamType};
use web3::types::{Address, BlockId, CallRequest, H160, U256, U64};

#[derive(Deserialize, Serialize)]
pub struct CreateTransaction {
    sender_address: String,
    receiver_address: String,
    from_token_address: String,
    to_token_address: String,
    origin_network: Option<Uuid>,
    destin_network: Option<Uuid>,
    asset_type: Option<Uuid>,
    transfer_amount: i64,
    bridge_fee: i64,
    tx_status: Option<Uuid>,
    created_by: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct RequestedTransaction {
    sender_address: String,
    receiver_address: String,
    from_token_address: Uuid,
    to_token_address: Uuid,
    origin_network: Option<Uuid>,
    destin_network: Option<Uuid>,
    asset_type: Option<Uuid>,
    transfer_amount: f64,
    created_by: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ResponseTransaction {
    id: Uuid,
    sender_address: String,
    receiver_address: String,
    transfer_amount: u64,
    gas_limit: String,
    max_priority_fee_per_gas: i64,
    max_fee_per_gas: u64,
}

#[derive(Deserialize, Serialize)]
pub struct TransactionHash {
    id: Uuid,
    hash: String,
}

pub async fn get_all_tx(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_txs = Transaction::get_all_tx(&data.db).await;
    if all_txs.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    let data = all_txs.unwrap();
    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });
    Ok(Json(json_response))
}

// handle tx validation
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
    // for native token type, get transaciton receipt from hash
    let tx_receipt = match get_tx(
        &data.db,
        transaction.origin_network.unwrap(),
        payload.hash.clone(),
    )
    .await
    {
        Ok(tx) => {
            println!("Origin Tx Receipt: {:#?}", tx);
            //validate from address
            match Some(tx.from) {
                Some(from_address) => {
                    if from_address != Some(Address::from_str(&transaction.sender_address).unwrap())
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
            tx
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
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
            for log in logs.into_iter() {
                println!("Log: {:#?}", log);
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
                // validate the value transfer
                let decimal = get_decimal(
                    &data.db,
                    transaction.origin_network.unwrap(),
                    Uuid::from_str(transaction.from_token_address.as_str()).unwrap(),
                )
                .await
                .expect("Error getting decimal");
                let decimal_factor = (10u128).pow(decimal.into());
                let value_int = value.clone().into_uint().unwrap();
                let division = value_int.div_mod(U256::from(decimal_factor));
                if division.0 != U256::from(transaction.transfer_amount) {
                    let json_response = serde_json::json!({
                        "status": "fail",
                        "data": format!("Value does not match")
                    });
                    return Ok(Json(json_response));
                }
            }
            let from_address = Address::from(topics[1]); //sender address
            let to_address = Address::from(topics[2]); //receiver address
            if from_address
                != Address::from_str(&transaction.sender_address).expect("Error decoding address")
            {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("To address does not match")
                });
                return Ok(Json(json_response));
            }
            let bridge = Bridge::get_bridge_info(
                &data.db,
                Uuid::from_str("1b013cc6-1f47-46a6-8954-04d85866708f").unwrap(),
            )
            .await
            .expect("Failed to get bridge information");
            if to_address
                != Address::from_str(&bridge.bridge_address).expect("Error decoding address")
            {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("To address does not match")
                });
                return Ok(Json(json_response));
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
    // validate confirmation block
    match get_confirmed_block(
        &data.db,
        transaction.origin_network.unwrap(),
        BlockId::Hash(tx_receipt.block_hash.unwrap()),
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
    // query network id from transaction id
    let network =
        match Network::get_network_by_id(&data.db, transaction.destin_network.unwrap()).await {
            Ok(network_id) => network_id,
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                return Ok(Json(json_response));
            }
        };
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
    //validate token amount in the pool
    match get_token_supply(&data.db, network.id, transaction.asset_type.unwrap()).await {
        Ok(total_supply) => {
            if total_supply.is_zero() {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Total Supply is zero")
                });
                return Ok(Json(json_response));
            } else {
                total_supply
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
    let p_k: String = dotenvy::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    if transaction.asset_type == Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()))
    {
        match send_raw_transaction(&data.db, network.id, &transaction, p_k.as_str()).await {
            Ok(tx) => {
                // get transaciton receipt from hash
                let new_tx_receipt = match get_tx(&data.db, payload.id, tx.to_string()).await {
                    Ok(tx) => tx,
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                };
                //insert destination hash to db
                match Transaction::update_tx_hash(&data.db, network.id, None, Some(tx.to_string()))
                    .await
                {
                    Ok(updated_tx) => {
                        println!("Update tx: {:?}", updated_tx);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
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
                                println!("Broadcast transaction: {:#?}", tx);
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": tx
                                });
                                Ok(Json(json_response))
                            }
                            Err(err) => {
                                let json_response = serde_json::json!({
                                    "status": "fail",
                                    "data": format!("Err: {}", err)
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
            }
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                Ok(Json(json_response))
            }
        }
    } else if transaction.asset_type
        == Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()))
    {
        match send_erc20_token(&data.db, network.id, &transaction, p_k.as_str()).await {
            Ok(tx) => {
                println!("Tx hash: {:#?}", tx);
                let new_tx_receipt = match get_tx(&data.db, network.id, format!("{:#?}", tx)).await
                {
                    Ok(tx) => tx,
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
                        });
                        return Ok(Json(json_response));
                    }
                };
                //insert destination hash to db
                match Transaction::update_tx_hash(&data.db, network.id, None, Some(tx.to_string()))
                    .await
                {
                    Ok(updated_tx) => {
                        println!("Update tx: {:?}", updated_tx);
                    }
                    Err(err) => {
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": format!("Err: {}", err)
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
                        println!("Broadcast transaction: {:#?}", &new_tx_receipt);
                        let json_response = serde_json::json!({
                            "status": "fail",
                            "data": &new_tx_receipt
                        });
                        Ok(Json(json_response))
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

pub async fn validate_tx(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<RequestedTransaction>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //validate request body
    fn generate_error_response(field_name: &str) -> Json<serde_json::Value> {
        let json_response = serde_json::json!({
            "status": "Request Body Failed",
            "data": format!("{} must be provided!", field_name)
        });
        Json(json_response)
    }
    if payload.origin_network.is_none() {
        return Ok(generate_error_response("Origin Network"));
    } else if payload.destin_network.is_none() {
        return Ok(generate_error_response("Destinated Network"));
    } else if payload.asset_type.is_none() {
        return Ok(generate_error_response("Asset Type"));
    } else if payload.created_by.is_none() {
        return Ok(generate_error_response("Creator"));
    } else if payload.transfer_amount <= 0.00 {
        //validate transfer amount greater than 0.00
        let json_response = serde_json::json!({
            "status": "Request Body Failed",
            "data": "Amount must greater than zero",
        });
        return Ok(Json(json_response));
    } else if payload.origin_network == payload.destin_network {
        let json_response = serde_json::json!({
            "status": "Fail",
            "data": "Same networks are not allowed",
        });
        return Ok(Json(json_response));
    }
    // validate sender account
    validate_account(
        &data.db,
        (payload.origin_network).unwrap(),
        Address::from_str((payload.sender_address).as_str()).unwrap(),
    )
    .await
    .unwrap_or_default();
    // validate receiver address
    validate_account(
        &data.db,
        (payload.destin_network).unwrap(),
        Address::from_str((payload.receiver_address).as_str()).unwrap(),
    )
    .await
    .unwrap_or_default();
    //validate origin network
    let validated_origin_network =
        match Network::get_network_by_id(&data.db, payload.origin_network.unwrap()).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error retrieving origin network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    //validate destination network
    let validated_destinated_network =
        match Network::get_network_by_id(&data.db, payload.destin_network.unwrap()).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error retrieving destin network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
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
    //query gas_price
    let current_gas_price = match get_gas_price(&data.db, (payload.origin_network).unwrap()).await {
        Ok(gas_price) => gas_price,
        Err(err) => {
            let error_message = format!("Error Gas Price: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    // println!("UUid NativeToken: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()));
    // println!("UUid ERC20Token: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()));
    // validate asset type
    if payload.asset_type != Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()))
        && payload.asset_type != Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()))
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
        payload.asset_type.unwrap(),
        validated_from_token.id,
        payload.transfer_amount,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error converting token value: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    let call_req = CallRequest {
        from: Some(H160::from_str(payload.sender_address.clone().as_str()).unwrap()),
        to: Some(H160::from_str(payload.receiver_address.clone().as_str()).unwrap()),
        gas: None,
        gas_price: Some(current_gas_price),
        value: Some(U256::from(transfer_value)),
        data: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    //estimated_gas_price and balance validation
    let est_gas_price =
        match get_est_gas_price(&data.db, (payload.origin_network).unwrap(), call_req).await {
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
    // Calculation of the bridge fee as needed
    let bridge = Bridge::get_bridge_info(
        &data.db,
        Uuid::from_str("1b013cc6-1f47-46a6-8954-04d85866708f").unwrap(),
    )
    .await
    .expect("ERROR: Failed to get bridge info");
    let bridge_fee = match token_converter(
        &data.db,
        validated_origin_network.id,
        payload.asset_type.unwrap(),
        validated_from_token.id,
        bridge.bridge_fee * payload.transfer_amount,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error retrieving est gas price: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    println!("Bridge fee: {}", bridge_fee);
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
    let inserted_tx = RequestInsertTx {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        from_token_address: validated_from_token.id.to_string(),
        to_token_address: validated_to_token.id.to_string(),
        origin_network: Some(validated_origin_network.id),
        destin_network: Some(validated_destinated_network.id),
        asset_type: payload.asset_type,
        transfer_amount: transfer_value as i64,
        bridge_fee: bridge_fee as i64,
        tx_status: Some(transaction_status_id),
        origin_tx_hash: None,
        destin_tx_hash: None,
        created_by: payload.created_by,
    };
    //insert unconfirmed tx to database
    let created_tx = match Transaction::create(&data.db, inserted_tx).await {
        Ok(tx) => {
            println!("{:#?}", tx);
            tx
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let base_fee = match get_base_fee(&data.db, validated_origin_network.id).await {
        Ok(block) => {
            println!("{:#?}", &block);
            match block.base_fee_per_gas {
                Some(base_gas_fee) => base_gas_fee,
                None => U256::from(0),
            }
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let response_tx = ResponseTransaction {
        id: created_tx.id,
        sender_address: payload.sender_address.clone(),
        receiver_address: bridge.bridge_address,
        transfer_amount: (U256::from(bridge_fee) + U256::from(transfer_value)).as_u64(),
        gas_limit: est_gas_price.to_string(),
        max_priority_fee_per_gas: 0,
        max_fee_per_gas: base_fee.as_u64(),
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": response_tx
    });
    Ok(Json(json_response))
}
