# \BatchesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**cancel_batch_v1_batches_batch_id_cancel_post**](BatchesApi.md#cancel_batch_v1_batches_batch_id_cancel_post) | **POST** /v1/batches/{batch_id}/cancel | Cancel Batch
[**create_batch_v1_batches_post**](BatchesApi.md#create_batch_v1_batches_post) | **POST** /v1/batches | Create Batch
[**list_batches_v1_batches_get**](BatchesApi.md#list_batches_v1_batches_get) | **GET** /v1/batches | List Batches
[**retrieve_batch_results_v1_batches_batch_id_results_get**](BatchesApi.md#retrieve_batch_results_v1_batches_batch_id_results_get) | **GET** /v1/batches/{batch_id}/results | Retrieve Batch Results
[**retrieve_batch_v1_batches_batch_id_get**](BatchesApi.md#retrieve_batch_v1_batches_batch_id_get) | **GET** /v1/batches/{batch_id} | Retrieve Batch



## cancel_batch_v1_batches_batch_id_cancel_post

> serde_json::Value cancel_batch_v1_batches_batch_id_cancel_post(batch_id, provider)
Cancel Batch

Cancel a batch.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** |  | [required] |
**provider** | **String** |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_batch_v1_batches_post

> serde_json::Value create_batch_v1_batches_post(create_batch_request)
Create Batch

Create a batch of LLM requests for asynchronous processing.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_batch_request** | [**CreateBatchRequest**](CreateBatchRequest.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_batches_v1_batches_get

> serde_json::Value list_batches_v1_batches_get(provider, after, limit)
List Batches

List batches for a provider.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider** | **String** |  | [required] |
**after** | Option<**String**> |  |  |
**limit** | Option<**i32**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## retrieve_batch_results_v1_batches_batch_id_results_get

> serde_json::Value retrieve_batch_results_v1_batches_batch_id_results_get(batch_id, provider)
Retrieve Batch Results

Retrieve the results of a completed batch.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** |  | [required] |
**provider** | **String** |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## retrieve_batch_v1_batches_batch_id_get

> serde_json::Value retrieve_batch_v1_batches_batch_id_get(batch_id, provider)
Retrieve Batch

Retrieve the status of a batch.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**batch_id** | **String** |  | [required] |
**provider** | **String** |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

