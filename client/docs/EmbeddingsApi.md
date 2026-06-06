# \EmbeddingsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_embedding_v1_embeddings_post**](EmbeddingsApi.md#create_embedding_v1_embeddings_post) | **POST** /v1/embeddings | Create Embedding



## create_embedding_v1_embeddings_post

> models::CreateEmbeddingResponse create_embedding_v1_embeddings_post(embedding_request)
Create Embedding

OpenAI-compatible embeddings endpoint.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**embedding_request** | [**EmbeddingRequest**](EmbeddingRequest.md) |  | [required] |

### Return type

[**models::CreateEmbeddingResponse**](CreateEmbeddingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

