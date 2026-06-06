# ResponsesRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**guardrails** | Option<[**Vec<models::GuardrailConfig>**](GuardrailConfig.md)> |  | [optional]
**input** | Option<**serde_json::Value**> |  | 
**max_tool_iterations** | Option<**i32**> |  | [optional]
**mcp_server_ids** | Option<**Vec<uuid::Uuid>**> |  | [optional]
**mcp_servers** | Option<[**Vec<models::McpServerConfig>**](McpServerConfig.md)> |  | [optional]
**model** | **String** |  | 
**stream** | Option<**bool**> |  | [optional][default to false]
**tools** | Option<**Vec<std::collections::HashMap<String, serde_json::Value>>**> |  | [optional]
**tools_header** | Option<**String**> |  | [optional]
**user** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


