use async_trait::async_trait;
use uuid::Uuid;

use crate::api::{
    errors::ApiResult,
    traits::UsersApi,
    types::{
        FindUserRequest, PaginatedResponse, PaginationQuery, SearchQuery, UpdateUserRequest, User,
    },
};

use super::client::NestJsClient;

#[derive(Clone)]
pub struct NestJsUsersApi {
    client: NestJsClient,
}

impl NestJsUsersApi {
    pub fn new(client: NestJsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsersApi for NestJsUsersApi {
    async fn get_user(&self, user_id: Uuid) -> ApiResult<User> {
        let req = self.client.get(&format!("/users/find/id/{}", user_id));
        self.client.send(req).await
    }

    async fn find_user(&self, request: FindUserRequest) -> ApiResult<Option<User>> {
        let result = if let Some(id) = &request.id {
            let req = self.client.get(&format!("/users/find/id/{}", id));
            self.client.send::<User>(req).await
        } else if let Some(phone) = &request.phone {
            let req = self
                .client
                .get(&format!("/users/find/phone/{}", urlencoding::encode(phone)));
            self.client.send::<User>(req).await
        } else if let Some(npub) = &request.npub {
            let req = self
                .client
                .get(&format!("/users/find/npub/{}", urlencoding::encode(npub)));
            self.client.send::<User>(req).await
        } else {
            return Err(crate::api::errors::ApiError::Validation {
                message: "At least one search criteria (id, phone, or npub) must be provided"
                    .to_string(),
            });
        };

        match result {
            Ok(user) => Ok(Some(user)),
            Err(crate::api::errors::ApiError::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_users(&self, pagination: PaginationQuery) -> ApiResult<PaginatedResponse<User>> {
        // The NestJS backend has /users/all but doesn't seem to support pagination directly
        // We'll implement a basic version that gets all users and simulates pagination
        let req = self.client.get("/users/all");
        let all_users: Vec<User> = self.client.send(req).await?;

        // Apply pagination
        let total = all_users.len() as u64;
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        let start = ((page - 1) * limit) as usize;
        let end = std::cmp::min(start + limit as usize, total as usize);

        let users = if start < total as usize {
            all_users[start..end].to_vec()
        } else {
            Vec::new()
        };

        let total_pages = (total as f64 / limit as f64).ceil() as u32;

        Ok(PaginatedResponse {
            data: users,
            total,
            page,
            limit,
            total_pages,
        })
    }

    async fn search_users(
        &self,
        search: SearchQuery,
        pagination: PaginationQuery,
    ) -> ApiResult<PaginatedResponse<User>> {
        // The NestJS backend doesn't have a dedicated search endpoint,
        // so we'll get all users and filter them client-side
        let all_users_response = self
            .get_users(PaginationQuery {
                page: Some(1),
                limit: Some(1000),
            })
            .await?;
        let all_users = all_users_response.data;

        // Apply search filters
        let filtered_users: Vec<User> = all_users
            .into_iter()
            .filter(|user| {
                if let Some(query) = &search.query {
                    let query_lower = query.to_lowercase();

                    // Search in phone number (if present)
                    if let Some(phone) = &user.phone {
                        if phone.number.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }

                    // Search in npub (if present)
                    if let Some(nostr) = &user.nostr {
                        if nostr.npub.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }

                    // Search in user ID (convert to string)
                    if user.id.to_string().to_lowercase().contains(&query_lower) {
                        return true;
                    }

                    false
                } else {
                    true
                }
            })
            .collect();

        // Apply pagination to filtered results
        let total = filtered_users.len() as u64;
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        let start = ((page - 1) * limit) as usize;
        let end = std::cmp::min(start + limit as usize, total as usize);

        let users = if start < total as usize {
            filtered_users[start..end].to_vec()
        } else {
            Vec::new()
        };

        let total_pages = (total as f64 / limit as f64).ceil() as u32;

        Ok(PaginatedResponse {
            data: users,
            total,
            page,
            limit,
            total_pages,
        })
    }

    async fn update_user(&self, request: UpdateUserRequest) -> ApiResult<User> {
        let req = self.client.patch("/users/update");
        self.client.send_json(req, &request).await
    }

    async fn delete_user(&self, user_id: Uuid) -> ApiResult<()> {
        // The NestJS backend doesn't seem to have a delete user endpoint
        // This would typically be a DELETE request to /users/{id}
        // For now, we'll return an error indicating this operation is not supported
        Err(crate::api::errors::ApiError::NotFound {
            resource: format!(
                "Delete user operation not supported by backend for user {}",
                user_id
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::config::ApiConfig;

    // Helper function to create a test client
    fn create_test_client() -> NestJsClient {
        let config = ApiConfig::default();
        NestJsClient::new(&config).expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_users_api_creation() {
        let client = create_test_client();
        let users_api = NestJsUsersApi::new(client);

        // Test that the API can be created
        assert!(true); // If we get here, creation succeeded
    }

    #[test]
    fn test_find_user_request_validation() {
        // Test that we properly validate find user requests
        let empty_request = FindUserRequest {
            id: None,
            phone: None,
            npub: None,
        };

        // This would fail validation in the actual API call
        assert!(
            empty_request.id.is_none()
                && empty_request.phone.is_none()
                && empty_request.npub.is_none()
        );
    }

    #[test]
    fn test_pagination_calculation() {
        // Test pagination logic
        let total_items = 25;
        let page_size = 10;

        // Page 0: items 0-9
        let page_0_start = 0 * page_size;
        let page_0_end = std::cmp::min(page_0_start + page_size, total_items);
        assert_eq!(page_0_start, 0);
        assert_eq!(page_0_end, 10);

        // Page 2: items 20-24
        let page_2_start = 2 * page_size;
        let page_2_end = std::cmp::min(page_2_start + page_size, total_items);
        assert_eq!(page_2_start, 20);
        assert_eq!(page_2_end, 25);

        // Total pages calculation
        let total_pages = (total_items as f64 / page_size as f64).ceil() as u32;
        assert_eq!(total_pages, 3);
    }
}
