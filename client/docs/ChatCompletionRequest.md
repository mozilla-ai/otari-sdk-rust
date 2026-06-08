# ChatCompletionRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**guardrails** | Option<[**Vec<models::GuardrailConfig>**](GuardrailConfig.md)> |  | [optional]
**max_completion_tokens** | Option<**i32**> |  | [optional]
**max_tokens** | Option<**i32**> |  | [optional]
**max_tool_iterations** | Option<**i32**> |  | [optional]
**mcp_server_ids** | Option<**Vec<uuid::Uuid>**> |  | [optional]
**mcp_servers** | Option<[**Vec<models::McpServerConfig>**](McpServerConfig.md)> |  | [optional]
**messages** | [**Vec<models::ChatMessageInput>**](ChatMessageInput.md) |  | 
**model** | **String** |  | 
**response_format** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**stream** | Option<**bool**> |  | [optional][default to false]
**stream_options** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**temperature** | Option<**f64**> |  | [optional]
**tool_choice** | Option<[**models::ToolChoice**](ToolChoice.md)> |  | [optional]
**tools** | Option<**Vec<std::collections::HashMap<String, serde_json::Value>>**> |  | [optional]
**tools_header** | Option<**String**> | Optional override for the lead-in that the gateway prepends before the per-tool hint block in the system message. Useful for expressing global tool-selection policy (e.g. 'prefer MCP tools over code_execution'). Falls back to GATEWAY_TOOLS_HEADER env, then to the built-in default. | [optional]
**top_p** | Option<**f64**> |  | [optional]
**user** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


