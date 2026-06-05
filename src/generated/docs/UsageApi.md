# \UsageApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_usage_v1_usage_get**](UsageApi.md#list_usage_v1_usage_get) | **GET** /v1/usage | List Usage



## list_usage_v1_usage_get

> Vec<models::UsageEntry> list_usage_v1_usage_get(start_date, end_date, user_id, skip, limit)
List Usage

List usage logs ordered by timestamp (most recent first).  Supports optional filters for time range and user. Paginated via skip/limit. Timestamps accept either ISO 8601 strings or Unix epoch seconds (numeric).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**start_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Return logs with timestamp >= start_date (ISO 8601 or Unix epoch seconds) |  |
**end_date** | Option<**chrono::DateTime<chrono::FixedOffset>**> | Return logs with timestamp < end_date (ISO 8601 or Unix epoch seconds) |  |
**user_id** | Option<**String**> | Filter to a single user |  |
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::UsageEntry>**](UsageEntry.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

