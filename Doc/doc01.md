# Sending a POST Request-Tx

This guide will walk you through sending a POST request in Postman. Before sending the request, you'll need to fetch specific token IDs and network IDs from different endpoints. Here's how to do it step by step:

## Prerequisites

Before you start, make sure you have Postman installed on your system.

## Step 1: Open Postman

Launch Postman on your computer.

## Step 2: Fetch Token IDs and Network IDs

Before creating the request body, you'll need to obtain the following Token IDs and network IDs from your API endpoints:

### Fetch `from_token_address` and `to_token_address` UUIDs

1. Create a new request in Postman (you can name it "Fetch Token Address UUIDs").

2. Choose the GET method.

3. Enter the URL: `http://127.0.0.1:8000/token_addresses` for fetching token address IDs.

4. Send the GET request.

5. Inspect the response to find your desirable IDs for `from_token_address` and `to_token_address`.

### Fetch `origin_network` and `destin_network` IDs

1. Create a new request in Postman (you can name it "Fetch Network IDs").

2. Choose the appropriate GET method.

3. Enter the URL: `http://127.0.0.1:8000/all_networks` for fetching network IDs.

4. Send the GET request.

5. Inspect the response to find the IDs for `origin_network` and `destin_network`.

### Fetch `from_asset_type` and `to_asset_type` UUIDs

1. Create a new request in Postman (you can name it "Fetch Asset Type UUIDs").

2. Choose the GET method.

3. Enter the URL: `http://127.0.0.1:8000/token-uuid-erc20` and `http://127.0.0.1:8000/token-uuid-native` for fetching asset type UUIDs.

4. Send the GET requests for both endpoints.

5. Inspect the responses to find the IDs for `from_asset_type` and `to_asset_type`.

## Step 3: Create a New Request

1. Click on the "New" button in the top left corner of the Postman window.

2. Select "Request" to create a new request.

3. Give your request a meaningful name.

4. Choose the HTTP method as "POST."

5. Enter the request URL: `127.0.0.1:8000/request-tx` in the URL field.

## Step 4: Set Request Headers ( No Headers Required For Now!!)

If your API requires specific headers, set them in the "Headers" tab. Common headers include "Content-Type" and "Authorization."

## Step 5: Define the Request Body

1. Go to the "Body" tab in your request.

2. Select the "raw" option.

3. Choose the data format you want to use (e.g., JSON).

4. In the request body, specify the fields, data types, and values as required. Include the token_address IDs and network IDs that you fetched in Step 2.

```json
{
  "sender_address": {
    "type": "string",
    "value": "0x5852231D8a00306A67DfB128AEd50c1573411d60"
  },
  "receiver_address": {
    "type": "string",
    "value": "0x33b6Cc6169a2Acea65b89DDD886125e04BE49CF6"
  },
  "from_token_address": {
    "type": "string",
    "value": "your_from_token_address_uuid_here"
  },
  "to_token_address": {
    "type": "string",
    "value": "your_to_token_address_uuid_here"
  },
  "origin_network": {
    "type": "string",
    "value": "your_origin_network_id_here"
  },
  "destin_network": {
    "type": "string",
    "value": "your_destin_network_id_here"
  },
  "from_asset_type": {
    "type": "string",
    "value": "your_from_asset_type_uuid_here"
  },
  "to_asset_type": {
    "type": "string",
    "value": "your_to_asset_type_uuid_here"
  },
  "transfer_amount": {
    "type": "float",
    "value": 0.00000000000005
  },
  "created_by": {
    "type": "string",
    "value": "1abacae0-59c1-496c-bc04-83ea721c2052"
  }
}

Request example:

```json
{
  "sender_address": "0x5852231D8a00306A67DfB128AEd50c1573411d60",
  "receiver_address": "0x33b6Cc6169a2Acea65b89DDD886125e04BE49CF6",
  "from_token_address": "your_from_token_address_uuid_here",
  "to_token_address": "your_to_token_address_uuid_here",
  "origin_network": "your_origin_network_id_here",
  "destin_network": "your_destin_network_id_here",
  "from_asset_type": "your_from_asset_type_uuid_here",
  "to_asset_type": "your_to_asset_type_uuid_here",
  "transfer_amount": 0.00000000000005,
  "created_by": "1abacae0-59c1-496c-bc04-83ea721c2052"
}
