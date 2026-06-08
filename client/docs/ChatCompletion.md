# ChatCompletion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**choices** | [**Vec<models::CcChoice>**](CCChoice.md) |  | 
**created** | **i32** |  | 
**model** | **String** |  | 
**object** | **Object** |  (enum: chat.completion) | 
**service_tier** | Option<**String**> | Filter models by provider name | [optional]
**system_fingerprint** | Option<**String**> | Filter models by provider name | [optional]
**usage** | Option<[**models::CcCompletionUsage**](CCCompletionUsage.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


