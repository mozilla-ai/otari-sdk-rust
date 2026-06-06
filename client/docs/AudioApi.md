# \AudioApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_speech_v1_audio_speech_post**](AudioApi.md#create_speech_v1_audio_speech_post) | **POST** /v1/audio/speech | Create Speech
[**create_transcription_v1_audio_transcriptions_post**](AudioApi.md#create_transcription_v1_audio_transcriptions_post) | **POST** /v1/audio/transcriptions | Create Transcription



## create_speech_v1_audio_speech_post

> serde_json::Value create_speech_v1_audio_speech_post(audio_speech_request)
Create Speech

OpenAI-compatible audio speech (TTS) endpoint.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**audio_speech_request** | [**AudioSpeechRequest**](AudioSpeechRequest.md) |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json, audio/L16, audio/aac, audio/flac, audio/mpeg, audio/opus, audio/wav

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_transcription_v1_audio_transcriptions_post

> serde_json::Value create_transcription_v1_audio_transcriptions_post(file, model, language, prompt, response_format, temperature, user)
Create Transcription

OpenAI-compatible audio transcription endpoint.  Authentication modes: - Master key + user field: Use specified user (must exist) - API key + user field: Use specified user (must exist) - API key without user field: Use virtual user created with API key

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**file** | **String** |  | [required] |
**model** | **String** |  | [required] |
**language** | Option<**String**> |  |  |
**prompt** | Option<**String**> |  |  |
**response_format** | Option<**String**> |  |  |
**temperature** | Option<**f64**> |  |  |
**user** | Option<**String**> |  |  |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

