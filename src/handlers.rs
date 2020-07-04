use crate::domain::channel::model::DraftChannel;
use crate::domain::channel::repository::ChannelRepository;
use crate::domain::video::repository::VideoRepository;
use crate::exception::*;
use crate::IApplication;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{http, reject, reply, Rejection, Reply};

#[derive(Serialize, Deserialize)]
struct Empty {}

pub async fn create_channel_handler(
    application: Arc<dyn IApplication + Send + Sync>,
    draft_channel: DraftChannel,
) -> Result<impl Reply, Rejection> {
    application
        .channel_repository()
        .create(draft_channel)
        .await
        .map_err(|e: anyhow::Error| reject::custom(WarpError(e)))?;

    Ok(reply::with_status(
        reply::json(&Empty {}),
        http::StatusCode::CREATED,
    ))
}

pub async fn list_video_handler(
    application: Arc<dyn IApplication + Send + Sync>,
    offset: i64,
    limit: i64,
) -> Result<impl Reply, Rejection> {
    let videos = application
        .video_repository()
        .list(limit, offset)
        .await
        .map_err(|e: anyhow::Error| reject::custom(WarpError(e)))?;

    Ok(reply::with_status(
        reply::json(&videos),
        http::StatusCode::OK,
    ))
}
