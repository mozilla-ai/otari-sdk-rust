# \BudgetsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_budget_v1_budgets_post**](BudgetsApi.md#create_budget_v1_budgets_post) | **POST** /v1/budgets | Create Budget
[**delete_budget_v1_budgets_budget_id_delete**](BudgetsApi.md#delete_budget_v1_budgets_budget_id_delete) | **DELETE** /v1/budgets/{budget_id} | Delete Budget
[**get_budget_v1_budgets_budget_id_get**](BudgetsApi.md#get_budget_v1_budgets_budget_id_get) | **GET** /v1/budgets/{budget_id} | Get Budget
[**list_budgets_v1_budgets_get**](BudgetsApi.md#list_budgets_v1_budgets_get) | **GET** /v1/budgets | List Budgets
[**update_budget_v1_budgets_budget_id_patch**](BudgetsApi.md#update_budget_v1_budgets_budget_id_patch) | **PATCH** /v1/budgets/{budget_id} | Update Budget



## create_budget_v1_budgets_post

> models::BudgetResponse create_budget_v1_budgets_post(create_budget_request)
Create Budget

Create a new budget.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_budget_request** | [**CreateBudgetRequest**](CreateBudgetRequest.md) |  | [required] |

### Return type

[**models::BudgetResponse**](BudgetResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_budget_v1_budgets_budget_id_delete

> delete_budget_v1_budgets_budget_id_delete(budget_id)
Delete Budget

Delete a budget.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**budget_id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_budget_v1_budgets_budget_id_get

> models::BudgetResponse get_budget_v1_budgets_budget_id_get(budget_id)
Get Budget

Get details of a specific budget.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**budget_id** | **String** |  | [required] |

### Return type

[**models::BudgetResponse**](BudgetResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_budgets_v1_budgets_get

> Vec<models::BudgetResponse> list_budgets_v1_budgets_get(skip, limit)
List Budgets

List all budgets with pagination.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**skip** | Option<**i32**> |  |  |[default to 0]
**limit** | Option<**i32**> |  |  |[default to 100]

### Return type

[**Vec<models::BudgetResponse>**](BudgetResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_budget_v1_budgets_budget_id_patch

> models::BudgetResponse update_budget_v1_budgets_budget_id_patch(budget_id, update_budget_request)
Update Budget

Update a budget.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**budget_id** | **String** |  | [required] |
**update_budget_request** | [**UpdateBudgetRequest**](UpdateBudgetRequest.md) |  | [required] |

### Return type

[**models::BudgetResponse**](BudgetResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

