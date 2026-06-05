# \UsersApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_user_v1_users_post**](UsersApi.md#create_user_v1_users_post) | **POST** /v1/users | Create User
[**delete_user_v1_users_user_id_delete**](UsersApi.md#delete_user_v1_users_user_id_delete) | **DELETE** /v1/users/{user_id} | Delete User
[**get_user_usage_v1_users_user_id_usage_get**](UsersApi.md#get_user_usage_v1_users_user_id_usage_get) | **GET** /v1/users/{user_id}/usage | Get User Usage
[**get_user_v1_users_user_id_get**](UsersApi.md#get_user_v1_users_user_id_get) | **GET** /v1/users/{user_id} | Get User
[**list_users_v1_users_get**](UsersApi.md#list_users_v1_users_get) | **GET** /v1/users | List Users
[**update_user_v1_users_user_id_patch**](UsersApi.md#update_user_v1_users_user_id_patch) | **PATCH** /v1/users/{user_id} | Update User



## create_user_v1_users_post

> models::UserResponse create_user_v1_users_post(create_user_request)
Create User

Create a new user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_user_request** | [**CreateUserRequest**](CreateUserRequest.md) |  | [required] |

### Return type

[**models::UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user_v1_users_user_id_delete

> delete_user_v1_users_user_id_delete(user_id)
Delete User

Delete a user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_usage_v1_users_user_id_usage_get

> Vec<models::UsageLogResponse> get_user_usage_v1_users_user_id_usage_get(user_id, skip, limit)
Get User Usage

Get usage history for a specific user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::UsageLogResponse>**](UsageLogResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_v1_users_user_id_get

> models::UserResponse get_user_v1_users_user_id_get(user_id)
Get User

Get details of a specific user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |

### Return type

[**models::UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_users_v1_users_get

> Vec<models::UserResponse> list_users_v1_users_get(skip, limit)
List Users

List all users with pagination.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::UserResponse>**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user_v1_users_user_id_patch

> models::UserResponse update_user_v1_users_user_id_patch(user_id, update_user_request)
Update User

Update a user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**user_id** | **String** |  | [required] |
**update_user_request** | [**UpdateUserRequest**](UpdateUserRequest.md) |  | [required] |

### Return type

[**models::UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

