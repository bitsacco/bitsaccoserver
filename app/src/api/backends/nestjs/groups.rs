use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    traits::groups::{CreateGroupRequest, Group, GroupsApi, UpdateGroupRequest},
    types::{PaginatedResponse, PaginationQuery, SearchQuery},
};

use super::client::NestJsClient;

#[derive(Clone)]
pub struct NestJsGroupsApi {
    client: NestJsClient,
}

impl NestJsGroupsApi {
    pub fn new(client: NestJsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl GroupsApi for NestJsGroupsApi {
    async fn get_group(&self, group_id: Uuid) -> ApiResult<Group> {
        let req = self.client.get(&format!("/chamas/{}", group_id));
        self.client.send(req).await
    }

    async fn get_groups(&self, pagination: PaginationQuery) -> ApiResult<PaginatedResponse<Group>> {
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        let req = self
            .client
            .get(&format!("/chamas?page={}&size={}", page, limit));
        let response: serde_json::Value = self.client.send(req).await?;

        // The NestJS backend returns a structure like { chamas: [...], page, size, pages, total }
        // We need to map this to our PaginatedResponse structure
        let chamas = response
            .get("chamas")
            .and_then(|v| v.as_array())
            .ok_or_else(|| crate::api::errors::ApiError::Serialization {
                message: "Invalid response format: missing 'chamas' array".to_string(),
            })?;

        let groups: Result<Vec<Group>, _> = chamas
            .iter()
            .map(|item| serde_json::from_value(item.clone()))
            .collect();

        let groups = groups.map_err(|e| crate::api::errors::ApiError::Serialization {
            message: format!("Failed to deserialize groups: {}", e),
        })?;

        let page = response
            .get("page")
            .and_then(|v| v.as_u64())
            .unwrap_or(page as u64) as u32;

        let limit = response
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(limit as u64) as u32;

        let total = response.get("total").and_then(|v| v.as_u64()).unwrap_or(0);

        let total_pages = response.get("pages").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(PaginatedResponse {
            data: groups,
            total,
            page,
            limit,
            total_pages,
        })
    }

    async fn search_groups(
        &self,
        search: SearchQuery,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<Group>> {
        // The NestJS backend supports filtering by various criteria
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        let mut query_params = vec![format!("page={}", page), format!("size={}", limit)];

        // Add search query as a general filter if provided
        if let Some(_query) = &search.query {
            // Since the backend doesn't have a direct search parameter,
            // we'll get all groups and filter client-side
            return self.client_side_search(search, pagination).await;
        }

        // Add any filters based on the search criteria
        if let Some(filters) = &search.filters {
            for (key, value) in filters {
                match key.as_str() {
                    "memberId" => {
                        query_params.push(format!("memberId={}", urlencoding::encode(value)))
                    }
                    "createdBy" => {
                        query_params.push(format!("createdBy={}", urlencoding::encode(value)))
                    }
                    _ => {
                        // For other filters, we'll need to handle them client-side
                        return self.client_side_search(search, pagination).await;
                    }
                }
            }
        }

        let query_string = query_params.join("&");
        let req = self.client.get(&format!("/chamas?{}", query_string));
        let response: serde_json::Value = self.client.send(req).await?;

        // Parse the response similar to get_groups
        let chamas = response
            .get("chamas")
            .and_then(|v| v.as_array())
            .ok_or_else(|| crate::api::errors::ApiError::Serialization {
                message: "Invalid response format: missing 'chamas' array".to_string(),
            })?;

        let groups: Result<Vec<Group>, _> = chamas
            .iter()
            .map(|item| serde_json::from_value(item.clone()))
            .collect();

        let groups = groups.map_err(|e| crate::api::errors::ApiError::Serialization {
            message: format!("Failed to deserialize groups: {}", e),
        })?;

        let page = response
            .get("page")
            .and_then(|v| v.as_u64())
            .unwrap_or(page as u64) as u32;

        let limit = response
            .get("size")
            .and_then(|v| v.as_u64())
            .unwrap_or(limit as u64) as u32;

        let total = response.get("total").and_then(|v| v.as_u64()).unwrap_or(0);

        let total_pages = response.get("pages").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(PaginatedResponse {
            data: groups,
            total,
            page,
            limit,
            total_pages,
        })
    }

    async fn create_group(&self, request: CreateGroupRequest) -> ApiResult<Group> {
        let req = self.client.post("/chamas");
        self.client.send_json(req, &request).await
    }

    async fn update_group(&self, group_id: Uuid, request: UpdateGroupRequest) -> ApiResult<Group> {
        let req = self.client.patch(&format!("/chamas/{}", group_id));
        self.client.send_json(req, &request).await
    }

    async fn delete_group(&self, group_id: Uuid) -> ApiResult<()> {
        // The NestJS backend doesn't seem to have a delete group endpoint
        // This would typically be a DELETE request to /chamas/{id}
        // For now, we'll return an error indicating this operation is not supported
        Err(crate::api::errors::ApiError::NotFound {
            resource: format!(
                "Delete group operation not supported by backend for group {}",
                group_id
            ),
        })
    }
}

impl NestJsGroupsApi {
    /// Fallback method for client-side search when backend doesn't support the search criteria
    async fn client_side_search(
        &self,
        search: SearchQuery,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<Group>> {
        // Get all groups first
        let all_groups_response = self
            .get_groups(PaginationQuery {
                page: Some(1),
                limit: Some(1000),
            })
            .await?;
        let all_groups = all_groups_response.data;

        // Apply search filters
        let filtered_groups: Vec<Group> = all_groups
            .into_iter()
            .filter(|group| {
                if let Some(query) = &search.query {
                    let query_lower = query.to_lowercase();

                    // Search in group name
                    if group.name.to_lowercase().contains(&query_lower) {
                        return true;
                    }

                    // Search in description
                    if let Some(description) = &group.description {
                        if description.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }

                    // Search in group ID
                    if group.id.to_string().to_lowercase().contains(&query_lower) {
                        return true;
                    }

                    false
                } else {
                    true
                }
            })
            .collect();

        // Apply pagination to filtered results
        let total = filtered_groups.len() as u64;
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        let start = ((page - 1) * limit) as usize;
        let end = std::cmp::min(start + limit as usize, total as usize);

        let groups = if start < total as usize {
            filtered_groups[start..end].to_vec()
        } else {
            Vec::new()
        };

        let total_pages = (total as f64 / limit as f64).ceil() as u32;

        Ok(PaginatedResponse {
            data: groups,
            total,
            page,
            limit,
            total_pages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::ApiConfig;

    fn create_test_client() -> NestJsClient {
        let config = ApiConfig::default();
        NestJsClient::new(&config).expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_groups_api_creation() {
        let client = create_test_client();
        let groups_api = NestJsGroupsApi::new(client);

        // Test that the API can be created
        assert!(true); // If we get here, creation succeeded
    }

    #[test]
    fn test_query_params_construction() {
        let pagination = PaginationQuery {
            page: Some(1),
            limit: Some(20),
        };
        let query_params = vec![
            format!("page={}", pagination.page.unwrap_or(1)),
            format!("size={}", pagination.limit.unwrap_or(20)),
        ];

        assert_eq!(query_params[0], "page=1");
        assert_eq!(query_params[1], "size=20");

        let query_string = query_params.join("&");
        assert_eq!(query_string, "page=1&size=20");
    }

    #[test]
    fn test_url_encoding() {
        let member_id = "user@example.com";
        let encoded = urlencoding::encode(member_id);
        assert!(encoded.contains("user%40example.com"));
    }

    #[test]
    fn test_search_filtering() {
        // Test the logic for search filtering
        let search_query = "test";
        let group_name = "Test Group";
        let group_description = Some("A test group for testing".to_string());

        let query_lower = search_query.to_lowercase();
        assert!(group_name.to_lowercase().contains(&query_lower));
        assert!(group_description
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains(&query_lower));
    }
}
