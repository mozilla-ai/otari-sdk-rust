# ChatMessageInput

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content** | **String** |  | 
**role** | **Role** |  (enum: function) | 
**name** | **String** |  | 
**audio** | Option<[**models::MsgAudio**](MSGAudio.md)> |  | [optional]
**function_call** | Option<[**models::MsgFunctionCall**](MSGFunctionCall.md)> |  | [optional]
**refusal** | Option<**String**> |  | [optional]
**tool_calls** | Option<[**Vec<models::ToolCallsInner>**](ToolCallsInner.md)> |  | [optional]
**tool_call_id** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


