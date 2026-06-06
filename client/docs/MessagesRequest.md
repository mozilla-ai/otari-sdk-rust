# MessagesRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cache_control** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**guardrails** | Option<[**Vec<models::GuardrailConfig>**](GuardrailConfig.md)> |  | [optional]
**max_tokens** | **i32** |  | 
**max_tool_iterations** | Option<**i32**> |  | [optional]
**mcp_server_ids** | Option<**Vec<uuid::Uuid>**> |  | [optional]
**mcp_servers** | Option<[**Vec<models::McpServerConfig>**](McpServerConfig.md)> |  | [optional]
**messages** | **Vec<std::collections::HashMap<String, serde_json::Value>>** |  | 
**metadata** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**model** | **String** |  | 
**stop_sequences** | Option<**Vec<String>**> |  | [optional]
**stream** | Option<**bool**> |  | [optional][default to false]
**system** | Option<[**models::System**](System.md)> |  | [optional]
**temperature** | Option<**f64**> |  | [optional]
**thinking** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**tool_choice** | Option<**std::collections::HashMap<String, serde_json::Value>**> |  | [optional]
**tools** | Option<**Vec<std::collections::HashMap<String, serde_json::Value>>**> |  | [optional]
**tools_header** | Option<**String**> |  | [optional]
**top_k** | Option<**i32**> |  | [optional]
**top_p** | Option<**f64**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


