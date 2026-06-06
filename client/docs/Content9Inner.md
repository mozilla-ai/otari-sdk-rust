# Content9Inner

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**citations** | Option<[**Vec<models::MrTextBlockCitationsInner>**](MRTextBlockCitationsInner.md)> |  | [optional]
**text** | **String** |  | 
**r#type** | **Type** |  (enum: text, thinking, redacted_thinking, tool_use, server_tool_use, web_search_tool_result, web_fetch_tool_result, code_execution_tool_result, bash_code_execution_tool_result, text_editor_code_execution_tool_result, tool_search_tool_result, container_upload) | 
**signature** | **String** |  | 
**thinking** | **String** |  | 
**data** | **String** |  | 
**id** | **String** |  | 
**caller** | Option<[**models::Caller**](Caller.md)> |  | [optional]
**input** | **std::collections::HashMap<String, serde_json::Value>** |  | 
**name** | **Name** |  (enum: web_search, web_fetch, code_execution, bash_code_execution, text_editor_code_execution, tool_search_tool_regex, tool_search_tool_bm25) | 
**content** | [**models::Content6**](Content6.md) |  | 
**tool_use_id** | **String** |  | 
**file_id** | **String** |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


