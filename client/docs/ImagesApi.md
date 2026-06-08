# \ImagesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_image_v1_images_generations_post**](ImagesApi.md#create_image_v1_images_generations_post) | **POST** /v1/images/generations | Create Image



## create_image_v1_images_generations_post

> serde_json::Value create_image_v1_images_generations_post(image_generation_request)
Create Image

OpenAI-compatible image generation endpoint.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**image_generation_request** | [**ImageGenerationRequest**](ImageGenerationRequest.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

