# MessageResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**container** | Option<[**models::MrContainer**](MRContainer.md)> |  | [optional]
**content** | [**Vec<models::Content9Inner>**](Content9Inner.md) |  | 
**model** | [**models::Model**](Model.md) |  | 
**role** | **Role** |  (enum: assistant) | 
**stop_details** | Option<[**models::MrRefusalStopDetails**](MRRefusalStopDetails.md)> |  | [optional]
**stop_reason** | Option<**StopReason**> |  (enum: end_turn, max_tokens, stop_sequence, tool_use, pause_turn, refusal) | [optional]
**stop_sequence** | Option<**String**> | Filter models by provider name | [optional]
**r#type** | **Type** |  (enum: message) | 
**usage** | [**models::MrUsage**](MRUsage.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


