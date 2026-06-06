# \ResponsesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_response_v1_responses_post**](ResponsesApi.md#create_response_v1_responses_post) | **POST** /v1/responses | Create Response



## create_response_v1_responses_post

> serde_json::Value create_response_v1_responses_post(responses_request)
Create Response

OpenAI-compatible Responses endpoint.  Supports MCP tool-use loops, sandboxed code execution, and SearXNG web_search in both standalone mode and platform mode. Platform-mode requests resolve credentials via the platform service and (for non-tool-loop requests) get multi-attempt fallback across the resolved route. Tool-loop requests collapse to a single attempt — once ``on_first_response`` lock-in plumbing lands across the codebase, a follow-up will enable pre-lock-in fallback for tool-loop requests too.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**responses_request** | [**ResponsesRequest**](ResponsesRequest.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

