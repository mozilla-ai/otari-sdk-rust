# RerankRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**documents** | **Vec<String>** | List of document strings to rerank | 
**max_tokens_per_doc** | Option<**i32**> | Per-document truncation limit | [optional]
**model** | **String** | Provider-prefixed model ID, e.g. 'cohere:rerank-v3.5' | 
**query** | **String** | The search query to rerank documents against | 
**top_n** | Option<**i32**> | Maximum number of results to return | [optional]
**user** | Option<**String**> | User ID for usage attribution | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


