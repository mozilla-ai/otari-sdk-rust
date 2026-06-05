# \KeysApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_key_v1_keys_post**](KeysApi.md#create_key_v1_keys_post) | **POST** /v1/keys | Create Key
[**delete_key_v1_keys_key_id_delete**](KeysApi.md#delete_key_v1_keys_key_id_delete) | **DELETE** /v1/keys/{key_id} | Delete Key
[**get_key_v1_keys_key_id_get**](KeysApi.md#get_key_v1_keys_key_id_get) | **GET** /v1/keys/{key_id} | Get Key
[**list_keys_v1_keys_get**](KeysApi.md#list_keys_v1_keys_get) | **GET** /v1/keys | List Keys
[**update_key_v1_keys_key_id_patch**](KeysApi.md#update_key_v1_keys_key_id_patch) | **PATCH** /v1/keys/{key_id} | Update Key



## create_key_v1_keys_post

> models::CreateKeyResponse create_key_v1_keys_post(create_key_request)
Create Key

Create a new API key.  Requires master key authentication.  If user_id is provided, the key will be associated with that user (creates user if it doesn't exist). If user_id is not provided, a new user will be created automatically and the key will be associated with it.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_key_request** | [**CreateKeyRequest**](CreateKeyRequest.md) |  | [required] |

### Return type

[**models::CreateKeyResponse**](CreateKeyResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_key_v1_keys_key_id_delete

> delete_key_v1_keys_key_id_delete(key_id)
Delete Key

Delete (revoke) an API key.  Requires master key authentication.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_key_v1_keys_key_id_get

> models::KeyInfo get_key_v1_keys_key_id_get(key_id)
Get Key

Get details of a specific API key.  Requires master key authentication.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key_id** | **String** |  | [required] |

### Return type

[**models::KeyInfo**](KeyInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_keys_v1_keys_get

> Vec<models::KeyInfo> list_keys_v1_keys_get(skip, limit)
List Keys

List all API keys.  Requires master key authentication.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::KeyInfo>**](KeyInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_key_v1_keys_key_id_patch

> models::KeyInfo update_key_v1_keys_key_id_patch(key_id, update_key_request)
Update Key

Update an API key.  Requires master key authentication.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**key_id** | **String** |  | [required] |
**update_key_request** | [**UpdateKeyRequest**](UpdateKeyRequest.md) |  | [required] |

### Return type

[**models::KeyInfo**](KeyInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

