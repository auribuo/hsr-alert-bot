use serenity::all::Command;
use serenity::all::{CacheHttp, CreateCommand};
use serenity::async_trait;

pub mod disable;
pub mod enable;
pub mod set_alert_channel;
pub mod set_alert_role;
pub mod subscribe;

#[async_trait]
pub trait CreateCommandVecExt {
    async fn global_register_all(self, cache_http: impl CacheHttp);
}

#[async_trait]
impl CreateCommandVecExt for Vec<CreateCommand> {
    async fn global_register_all(self, cache_http: impl CacheHttp) {
        for cmd in self {
            Command::create_global_command(&cache_http, cmd)
                .await
                .unwrap();
        }
    }
}
