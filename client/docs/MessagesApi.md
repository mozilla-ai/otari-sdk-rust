# \MessagesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**count_message_tokens_v1_messages_count_tokens_post**](MessagesApi.md#count_message_tokens_v1_messages_count_tokens_post) | **POST** /v1/messages/count_tokens | Count Message Tokens
[**create_message_v1_messages_post**](MessagesApi.md#create_message_v1_messages_post) | **POST** /v1/messages | Create Message



## count_message_tokens_v1_messages_count_tokens_post

> models::CountTokensResponse count_message_tokens_v1_messages_count_tokens_post(count_tokens_request)
Count Message Tokens

Anthropic ``/v1/messages/count_tokens``-compatible endpoint.  Returns ``{\"input_tokens\": N}`` without contacting an upstream provider: counting is local, so there is no budget reservation, pricing, or usage logging. Authentication mirrors :func:`create_message` — platform mode resolves the caller's token against the platform, standalone mode validates the API key — so the endpoint is not an open token-counting oracle.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**count_tokens_request** | [**CountTokensRequest**](CountTokensRequest.md) |  | [required] |

### Return type

[**models::CountTokensResponse**](CountTokensResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_message_v1_messages_post

> models::MessageResponse create_message_v1_messages_post(messages_request)
Create Message

Anthropic Messages API-compatible endpoint.  Supports MCP tool-use loops, sandboxed code execution, and SearXNG web_search in both standalone mode and platform mode. Platform-mode requests resolve credentials via the platform service and (for non-tool-loop requests) get multi-attempt fallback across the resolved route. Tool-loop requests collapse to a single attempt — once ``on_first_response`` lock-in plumbing lands across the codebase, a follow-up will enable pre-lock-in fallback for tool-loop requests too.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**messages_request** | [**MessagesRequest**](MessagesRequest.md) |  | [required] |

### Return type

[**models::MessageResponse**](MessageResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

