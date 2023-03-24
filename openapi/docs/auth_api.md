# auth_api

All URIs are relative to _http://localhost_

| Method                                            | HTTP request                          | Description |
| ------------------------------------------------- | ------------------------------------- | ----------- |
| **forget_password**](auth_api.md#forget_password) | **POST** /api/v1/auth/forget_password |
| **get_auth_status**](auth_api.md#get_auth_status) | **GET** /api/v1/auth/status           |
| **reset_password**](auth_api.md#reset_password)   | **POST** /api/v1/auth/reset_password  |
| **signin**](auth_api.md#signin)                   | **POST** /api/v1/auth/signin          |
| **signout**](auth_api.md#signout)                 | **POST** /api/v1/auth/signout         |
| **signup**](auth_api.md#signup)                   | **POST** /api/v1/auth/signup          |
| **signup_finish**](auth_api.md#signup_finish)     | **POST** /api/v1/auth/signup/finish   |

# **forget_password**

> models::StatusOk forget_password(forget_password_request)

### Required Parameters

| Name                        | Type                                                  | Description | Notes |
| --------------------------- | ----------------------------------------------------- | ----------- | ----- |
| **forget_password_request** | [**ForgetPasswordRequest**](ForgetPasswordRequest.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **get_auth_status**

> models::StatusOk get_auth_status()

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **reset_password**

> models::StatusOk reset_password(reset_password_request)

### Required Parameters

| Name                       | Type                                                | Description | Notes |
| -------------------------- | --------------------------------------------------- | ----------- | ----- |
| **reset_password_request** | [**ResetPasswordRequest**](ResetPasswordRequest.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **signin**

> models::StatusOk signin(signin_request)

### Required Parameters

| Name               | Type                                  | Description | Notes |
| ------------------ | ------------------------------------- | ----------- | ----- |
| **signin_request** | [**SigninRequest**](SigninRequest.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **signout**

> models::StatusOk signout()

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **signup**

> models::StatusOk signup(sign_up_request)

### Required Parameters

| Name                | Type                                  | Description | Notes |
| ------------------- | ------------------------------------- | ----------- | ----- |
| **sign_up_request** | [**SignUpRequest**](SignUpRequest.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **signup_finish**

> models::StatusOk signup_finish(sign_up_finish_request)

### Required Parameters

| Name                       | Type                                              | Description | Notes |
| -------------------------- | ------------------------------------------------- | ----------- | ----- |
| **sign_up_finish_request** | [**SignUpFinishRequest**](SignUpFinishRequest.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
