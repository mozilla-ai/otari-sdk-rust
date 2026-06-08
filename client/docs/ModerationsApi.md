# \ModerationsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_moderation_v1_moderations_post**](ModerationsApi.md#create_moderation_v1_moderations_post) | **POST** /v1/moderations | Create Moderation



## create_moderation_v1_moderations_post

> models::ModerationResponse create_moderation_v1_moderations_post(moderation_request, include_raw)
Create Moderation

OpenAI-compatible moderations endpoint.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**moderation_request** | [**ModerationRequest**](ModerationRequest.md) |  | [required] |
**include_raw** | Option<**bool**> |  |  |[default to false]

### Return type

[**models::ModerationResponse**](ModerationResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

