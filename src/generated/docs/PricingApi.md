# \PricingApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_pricing_v1_pricing_model_key_delete**](PricingApi.md#delete_pricing_v1_pricing_model_key_delete) | **DELETE** /v1/pricing/{model_key} | Delete Pricing
[**get_pricing_history_v1_pricing_model_key_history_get**](PricingApi.md#get_pricing_history_v1_pricing_model_key_history_get) | **GET** /v1/pricing/{model_key}/history | Get Pricing History
[**get_pricing_v1_pricing_model_key_get**](PricingApi.md#get_pricing_v1_pricing_model_key_get) | **GET** /v1/pricing/{model_key} | Get Pricing
[**list_pricing_v1_pricing_get**](PricingApi.md#list_pricing_v1_pricing_get) | **GET** /v1/pricing | List Pricing
[**set_pricing_v1_pricing_post**](PricingApi.md#set_pricing_v1_pricing_post) | **POST** /v1/pricing | Set Pricing



## delete_pricing_v1_pricing_model_key_delete

> delete_pricing_v1_pricing_model_key_delete(model_key, effective_at)
Delete Pricing

Delete pricing entries for a model.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**model_key** | **String** |  | [required] |
**effective_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | ISO datetime identifying a specific pricing row to delete |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pricing_history_v1_pricing_model_key_history_get

> Vec<models::PricingResponse> get_pricing_history_v1_pricing_model_key_history_get(model_key)
Get Pricing History

Return the full pricing history for a model.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**model_key** | **String** |  | [required] |

### Return type

[**Vec<models::PricingResponse>**](PricingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pricing_v1_pricing_model_key_get

> models::PricingResponse get_pricing_v1_pricing_model_key_get(model_key, as_of)
Get Pricing

Get pricing for a specific model as of a timestamp.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**model_key** | **String** |  | [required] |
**as_of** | Option<**chrono::DateTime<chrono::FixedOffset>**> | ISO datetime for effective lookup |  |

### Return type

[**models::PricingResponse**](PricingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_pricing_v1_pricing_get

> Vec<models::PricingResponse> list_pricing_v1_pricing_get(skip, limit)
List Pricing

List all model pricing.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::PricingResponse>**](PricingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_pricing_v1_pricing_post

> models::PricingResponse set_pricing_v1_pricing_post(set_pricing_request)
Set Pricing

Set or update pricing for a model.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**set_pricing_request** | [**SetPricingRequest**](SetPricingRequest.md) |  | [required] |

### Return type

[**models::PricingResponse**](PricingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

