use anyhow::Result;
use sqlx::{prelude::FromRow, query, Sqlite, Transaction};

use crate::DbConn;

#[derive(Debug, PartialEq, FromRow)]
pub struct ServerSettingDAO {
    security_allowed_register_domain_list: Option<String>,
    pub security_disable_client_side_telemetry: bool,
    pub network_external_url: String,
}

const SERVER_SETTING_ROW_ID: i32 = 1;

impl ServerSettingDAO {
    pub fn security_allowed_register_domain_list(&self) -> impl IntoIterator<Item = &str> {
        self.security_allowed_register_domain_list
            .iter()
            .flat_map(|s| s.split(','))
            .filter(|s| !s.is_empty())
    }
}

impl DbConn {
    async fn internal_read_server_setting(
        &self,
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Result<Option<ServerSettingDAO>> {
        let setting: Option<ServerSettingDAO> = sqlx::query_as("SELECT security_disable_client_side_telemetry, network_external_url, security_allowed_register_domain_list FROM server_setting WHERE id = ?;")
            .bind(SERVER_SETTING_ROW_ID)
            .fetch_optional(&mut **transaction)
            .await?;
        Ok(setting)
    }

    pub async fn read_server_setting(&self) -> Result<ServerSettingDAO> {
        let mut transaction = self.pool.begin().await?;
        let setting = self.internal_read_server_setting(&mut transaction).await?;
        let Some(setting) = setting else {
            query!(
                "INSERT INTO server_setting (id) VALUES (?);",
                SERVER_SETTING_ROW_ID
            )
            .execute(&mut *transaction)
            .await?;
            let setting = self
                .internal_read_server_setting(&mut transaction)
                .await?
                .expect("Freshly-written row must always be present");
            transaction.commit().await?;
            return Ok(setting);
        };
        Ok(setting)
    }
}
