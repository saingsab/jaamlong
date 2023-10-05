# Sending a POST Request-Tx

This guide will walk you through sending a POST request in Postman. Before sending the request, you'll need to fetch specific token IDs and network IDs from different endpoints. Here's how to do it step by step:

## Prerequisites

Add necessary variable to .env file. See .env.example necessary variables

## Authentication Setup

1. **Make up the user account:**
   - Inside the '.env' file, fill in the 
    </br>SECRET_KEY,
    </br>USER_ID (Uuid),
    </br>USERNAME,
    </br>PASSWORD,
    </br>\\with anything for now! (Will implement registration later)

2. **Get Authentication Token:**
   - As an admin, log in to the system to obtain an authentication token.
Send POST request to http://127.0.0.1:7000/hsm/log-in with raw json body of 
```
{
    "username": "test",
    "password": "123"
}
```
3. **Store the Token:**
Response from log in request:
```
{
    "data": "success",
    "token": "AUTH_KEY"
}
```
   - Copy the obtained token which is the AUTH_KEY.

4. **Update .env file:**
   - Open the `.env` file and set the token using the key `JWT_TOKEN`.
   - Set the `PRIVATE_KEY` for bridge wallet
   - Set the `PATH` for HSM path

## Broadcast Request Setup
Send POST request to 127.0.0.1:8000/broadcast-tx with raw json body of 
```
{
    "id": "tx_id",
    "hash": "tx_hash_after_perform_transaction"
}
```
## Expected Response
```
{
    "data": {
        "Transaction ID": "tx_id",
        "Transaction hash": "broadcast",
        "Transaction status": "broadcast_status"
    },
    "status": "status"
}
```