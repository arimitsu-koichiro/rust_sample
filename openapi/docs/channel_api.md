# channel_api

All URIs are relative to _http://localhost_

| Method                                                   | HTTP request                                | Description |
| -------------------------------------------------------- | ------------------------------------------- | ----------- |
| **channel_cocket**](channel_api.md#channel_cocket)       | **GET** /api/v1/channel/{channel_id}/socket |
| **publish_channel**](channel_api.md#publish_channel)     | **POST** /api/v1/channel/{channel_id}       |
| **subscribe_channel**](channel_api.md#subscribe_channel) | **GET** /api/v1/channel/{channel_id}        |

# **channel_cocket**

> models::StatusOk channel_cocket(channel_id)

### Required Parameters

| Name           | Type       | Description | Notes |
| -------------- | ---------- | ----------- | ----- |
| **channel_id** | **String** |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **publish_channel**

> models::StatusOk publish_channel(channel_id, channel_message)

### Required Parameters

| Name                | Type                                    | Description | Notes |
| ------------------- | --------------------------------------- | ----------- | ----- |
| **channel_id**      | **String**                              |             |
| **channel_message** | [**ChannelMessage**](ChannelMessage.md) |             |

### Return type

[**models::StatusOk**](StatusOk.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **subscribe_channel**

> models::ChannelMessage subscribe_channel(channel_id)

### Required Parameters

| Name           | Type       | Description | Notes |
| -------------- | ---------- | ----------- | ----- |
| **channel_id** | **String** |             |

### Return type

[**models::ChannelMessage**](ChannelMessage.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
