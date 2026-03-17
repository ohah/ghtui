use ghtui_core::types::*;

use crate::client::GithubClient;
use crate::error::ApiError;

impl GithubClient {
    pub async fn list_notifications(
        &self,
        filters: &NotificationFilters,
    ) -> Result<Vec<Notification>, ApiError> {
        let mut params = Vec::new();

        if filters.all {
            params.push("all=true".to_string());
        }
        if filters.participating {
            params.push("participating=true".to_string());
        }

        let path = if params.is_empty() {
            "/notifications".to_string()
        } else {
            format!("/notifications?{}", params.join("&"))
        };

        let body = self.get(&path).await?;
        let notifications: Vec<Notification> = serde_json::from_str(&body)?;
        Ok(notifications)
    }

    pub async fn mark_notification_read(&self, thread_id: &str) -> Result<(), ApiError> {
        let path = format!("/notifications/threads/{}", thread_id);
        let url = self.url(&path);
        let response = self.http.patch(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await?;
            return Err(ApiError::GitHub {
                status,
                message: text,
            });
        }

        Ok(())
    }

    pub async fn mark_all_notifications_read(&self) -> Result<(), ApiError> {
        let body = serde_json::json!({});
        self.put("/notifications", &body).await?;
        Ok(())
    }

    pub async fn unsubscribe_thread(&self, thread_id: &str) -> Result<(), ApiError> {
        let path = format!("/notifications/threads/{}/subscription", thread_id);
        self.delete(&path).await?;
        Ok(())
    }

    pub async fn mark_thread_done(&self, thread_id: &str) -> Result<(), ApiError> {
        let path = format!("/notifications/threads/{}", thread_id);
        self.delete(&path).await?;
        Ok(())
    }
}
