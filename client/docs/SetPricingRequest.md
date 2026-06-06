# SetPricingRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**effective_at** | Option<**chrono::DateTime<chrono::FixedOffset>**> | ISO 8601 datetime from which this price applies. Defaults to now if omitted. | [optional]
**input_price_per_million** | **f64** | Price per 1M input tokens | 
**model_key** | **String** | Model identifier in format 'provider:model' | 
**output_price_per_million** | **f64** | Price per 1M output tokens | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


