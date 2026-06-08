# CcChatCompletionMessage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content** | Option<**String**> |  | [optional]
**refusal** | Option<**String**> |  | [optional]
**role** | **Role** |  (enum: assistant) | 
**annotations** | Option<**Vec<std::collections::HashMap<String, serde_json::Value>>**> |  | [optional]
**audio** | Option<[**models::CcChatCompletionAudio**](CCChatCompletionAudio.md)> |  | [optional]
**function_call** | Option<[**models::CcFunctionCall**](CCFunctionCall.md)> |  | [optional]
**tool_calls** | Option<[**Vec<models::CcChatCompletionMessageToolCallsInner>**](CCChatCompletionMessageToolCallsInner.md)> |  | [optional]
**reasoning** | Option<[**models::CckReasoning**](CCKReasoning.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


