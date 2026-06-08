# \HealthApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**health_check_health_get**](HealthApi.md#health_check_health_get) | **GET** /health | Health Check
[**health_liveness_health_liveness_get**](HealthApi.md#health_liveness_health_liveness_get) | **GET** /health/liveness | Health Liveness
[**health_readiness_health_readiness_get**](HealthApi.md#health_readiness_health_readiness_get) | **GET** /health/readiness | Health Readiness



## health_check_health_get

> std::collections::HashMap<String, String> health_check_health_get()
Health Check

General health check endpoint.  Returns basic health status. For infrastructure monitoring, use /health/readiness or /health/liveness instead.

### Parameters

This endpoint does not need any parameter.

### Return type

**std::collections::HashMap<String, String>**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## health_liveness_health_liveness_get

> String health_liveness_health_liveness_get()
Health Liveness

Liveness probe endpoint.  Simple check to verify the process is alive and responding. Used by Kubernetes/container orchestrators for liveness probes.  Returns:     Plain text \"I'm alive!\" message

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## health_readiness_health_readiness_get

> std::collections::HashMap<String, serde_json::Value> health_readiness_health_readiness_get()
Health Readiness

Readiness probe endpoint.  Checks if the gateway is ready to serve requests by validating: - Database connectivity - Service availability  Used by Kubernetes/container orchestrators for readiness probes. Returns HTTP 503 if any dependency is unavailable.  Returns:     dict: Status object with health details  Raises:     HTTPException: 503 if service is not ready

### Parameters

This endpoint does not need any parameter.

### Return type

[**std::collections::HashMap<String, serde_json::Value>**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

