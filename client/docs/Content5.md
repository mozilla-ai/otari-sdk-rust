# Content5

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error_code** | **ErrorCode** |  (enum: invalid_tool_input, unavailable, too_many_requests, execution_time_exceeded, file_not_found) | 
**error_message** | Option<**String**> |  | [optional]
**r#type** | **Type** |  (enum: text_editor_code_execution_tool_result_error, text_editor_code_execution_view_result, text_editor_code_execution_create_result, text_editor_code_execution_str_replace_result) | 
**content** | **String** |  | 
**file_type** | **FileType** |  (enum: text, image, pdf) | 
**num_lines** | Option<**i32**> |  | [optional]
**start_line** | Option<**i32**> |  | [optional]
**total_lines** | Option<**i32**> |  | [optional]
**is_file_update** | **bool** |  | 
**lines** | Option<**Vec<String>**> |  | [optional]
**new_lines** | Option<**i32**> |  | [optional]
**new_start** | Option<**i32**> |  | [optional]
**old_lines** | Option<**i32**> |  | [optional]
**old_start** | Option<**i32**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


