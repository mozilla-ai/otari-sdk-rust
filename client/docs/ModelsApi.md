# \ModelsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_model_v1_models_model_id_get**](ModelsApi.md#get_model_v1_models_model_id_get) | **GET** /v1/models/{model_id} | Get Model
[**list_models_v1_models_get**](ModelsApi.md#list_models_v1_models_get) | **GET** /v1/models | List Models



## get_model_v1_models_model_id_get

> models::ModelObject get_model_v1_models_model_id_get(model_id)
Get Model

Get details for a specific model.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**model_id** | **String** |  | [required] |

### Return type

[**models::ModelObject**](ModelObject.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_models_v1_models_get

> models::ModelListResponse list_models_v1_models_get(provider)
List Models

List all available models.  Returns models auto-discovered from configured providers, enriched with pricing data from the model_pricing table when available. Models that only exist in the pricing table are also included for backward compatibility.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**provider** | Option<**String**> | Filter models by provider name |  |

### Return type

[**models::ModelListResponse**](ModelListResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

