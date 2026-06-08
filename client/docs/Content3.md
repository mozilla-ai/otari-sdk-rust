# Content3

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_code** | **ErrorCode** |  (enum: invalid_tool_input, unavailable, too_many_requests, execution_time_exceeded, output_file_too_large) | 
**r#type** | **Type** |  (enum: bash_code_execution_tool_result_error, bash_code_execution_result) | 
**content** | [**Vec<models::MrBashCodeExecutionOutputBlock>**](MRBashCodeExecutionOutputBlock.md) |  | 
**return_code** | **i32** |  | 
**stderr** | **String** |  | 
**stdout** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


