# Content4

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_code** | **ErrorCode** |  (enum: invalid_tool_input, unavailable, too_many_requests, execution_time_exceeded) | 
**r#type** | **Type** |  (enum: code_execution_tool_result_error, code_execution_result, encrypted_code_execution_result) | 
**content** | [**Vec<models::MrCodeExecutionOutputBlock>**](MRCodeExecutionOutputBlock.md) |  | 
**return_code** | **i32** |  | 
**stderr** | **String** |  | 
**stdout** | **String** |  | 
**encrypted_stdout** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


