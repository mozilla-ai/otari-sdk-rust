# \ChatApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**chat_completions_v1_chat_completions_post**](ChatApi.md#chat_completions_v1_chat_completions_post) | **POST** /v1/chat/completions | Chat Completions



## chat_completions_v1_chat_completions_post

> models::ChatCompletion chat_completions_v1_chat_completions_post(chat_completion_request)
Chat Completions

OpenAI-compatible chat completions endpoint.  Supports both streaming and non-streaming responses. Handles reasoning content from otari providers.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**chat_completion_request** | [**ChatCompletionRequest**](ChatCompletionRequest.md) |  | [required] |

### Return type

[**models::ChatCompletion**](ChatCompletion.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

