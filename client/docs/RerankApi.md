# \RerankApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_rerank_v1_rerank_post**](RerankApi.md#create_rerank_v1_rerank_post) | **POST** /v1/rerank | Create Rerank



## create_rerank_v1_rerank_post

> models::RerankResponse create_rerank_v1_rerank_post(rerank_request)
Create Rerank

Rerank documents by relevance to a query.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**rerank_request** | [**RerankRequest**](RerankRequest.md) |  | [required] |

### Return type

[**models::RerankResponse**](RerankResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

