use std::sync::Arc;

use axum::{
    body::Bytes,
    debug_handler,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_auth::AuthBasic;
use serde::Deserialize;
use thiserror::Error;
use utoipa::{OpenApi, ToSchema};

use crate::{
    guest::{Guest, GID},
    node::{NodeData, NodeID},
    soul::Soul,
    world::World,
};

pub mod error {
    use super::*;

    #[derive(Debug, Error, ToSchema)]
    pub enum ApiError {
        #[error("auth error, wrong uid::{0} or password")]
        #[schema(example = "Wrong uid or password.")]
        AuthError(String),

        #[error("no password found in request header")]
        #[schema(example = "No password were given.")]
        EmptyPassword,

        /// all the other error is from backend
        #[error(transparent)]
        #[schema(value_type = Object, example = "Error from internal.")]
        BackendError(#[from] anyhow::Error),
    }

    impl IntoResponse for ApiError {
        fn into_response(self) -> Response {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")).into_response()
        }
    }
}

type Result<T> = std::result::Result<T, error::ApiError>;

async fn verify_soul(world: &World, uid: &String, pw_hash: Option<String>) -> Result<()> {
    // Verify soul authority
    let pw_hash = pw_hash.ok_or(error::ApiError::EmptyPassword)?;
    if !world.verify_soul(uid, &pw_hash).await? {
        Err(error::ApiError::AuthError(uid.clone()).into())
    } else {
        Ok(())
    }
}

pub mod soul {
    use super::*;

    #[derive(Debug, Deserialize, ToSchema)]
    pub struct RegisterInfo {
        name: String,
        pw_hash: String,
    }

    #[utoipa::path(
        post,
        path = "/register",
        request_body = RegisterInfo,
        responses(
            (status = 200, description = "Soul creat successfully.", body = Soul)
        )
    )]
    pub(crate) async fn register(
        State(world): State<Arc<World>>,
        Json(RegisterInfo { name, pw_hash }): Json<RegisterInfo>,
    ) -> Result<Json<Soul>> {
        Ok(Json(world.register_soul(name, pw_hash).await?))
    }

    #[utoipa::path(
        get,
        path = "/soul",
        responses(
            (status = 200, description = "Get soul info successfully.", body = Soul, example = json!(
                {
                    "name": "Test Name",
                    "uid": "6wQxPDN7sjDDyn92It5Q8w",
                    "pw_hash": "password",
                    "guest_quota": 1,
                    "guests": [
                        10086
                    ]
                }
            )),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn get(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Soul>> {
        verify_soul(&world, &uid, pw_hash).await?;
        Ok(Json(world.get_soul(&uid).await?.unwrap()))
    }
}

pub mod guest {
    use super::*;

    #[utoipa::path(
        get,
        path = "/guest/contain",
        responses(
            (status = 200, description = "Check whether guest is contained in soul.", body = bool),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn contain(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(id): Query<GID>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<bool>> {
        // Verify soul authority
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(wondering_soul.contain_guest(id)))
    }

    #[utoipa::path(
        get,
        path = "/guest",
        responses(
            (status = 200, description = "Get guest value.", body = Option<Guest>),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn get(
        AuthBasic((uid, pw_hash)): AuthBasic,
        Query(id): Query<GID>,
        State(world): State<Arc<World>>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap(); // Soul 注册后不会删除，所以这里不会出现 None
        Ok(Json(wondering_soul.get_guest(id).await?))
    }

    #[derive(Debug, Deserialize, ToSchema)]
    pub struct WalkCommand {
        #[schema(value_type = u64)]
        id: GID,
        #[schema(maximum = 1, minimum = -1)]
        x: i8,
        #[schema(maximum = 1, minimum = -1)]
        y: i8,
    }

    #[utoipa::path(
        post,
        path = "/guest/walk",
        request_body = WalkCommand,
        responses(
            (status = 200, description = "Drive the target guest to walk to a direction.", body = Option<Guest>),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn walk(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(WalkCommand { id, x, y }): Json<WalkCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.walk(id, (x as i16, y as i16)).await?))
    }

    #[derive(Debug, Deserialize, ToSchema)]
    pub struct HarvestCommand {
        #[schema(example = "Target an guest to do this command.")]
        id: GID,
        #[schema(
            example = "Harvest the node **at** the index.",
            maximum = 1023,
            minimum = 0
        )]
        at: usize,
    }

    #[utoipa::path(
        post,
        path = "/guest/harvest",
        request_body = HarvestCommand,
        responses(
            (status = 200, description = "Let the target guest harvest at given index of the node.", body = Option<Guest>),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn harvest(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(HarvestCommand { id, at }): Json<HarvestCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.harvest(id, at).await?))
    }

    #[derive(Debug, Deserialize, ToSchema)]
    pub struct HeatCommand {
        #[schema(example = "Target an guest to do this command.")]
        id: GID,
        #[schema(
            example = "Harvest the node **at** the index.",
            maximum = 1023,
            minimum = 0
        )]
        at: usize,
        #[schema(example = "Limit the maxium energy that would use to heat.")]
        energy: u8,
    }

    #[utoipa::path(
        post,
        path = "/guest/heat",
        request_body = HarvestCommand,
        responses(
            (status = 200, description = "Let the target guest heat at given index of the node.", body = Option<Guest>),
        ),
        security(
            ("http" = ["Basic"])
        )
    )]
    pub(crate) async fn heat(
        AuthBasic((uid, pw_hash)): AuthBasic,
        State(world): State<Arc<World>>,
        Json(HeatCommand { id, at, energy }): Json<HeatCommand>,
    ) -> Result<Json<Option<Guest>>> {
        verify_soul(&world, &uid, pw_hash).await?;

        let wondering_soul = world.get_wondering_soul(&uid).await?.unwrap();
        Ok(Json(wondering_soul.heat(id, at, energy).await?))
    }
}

// Node
pub mod node {
    use super::*;

    #[utoipa::path(
        get,
        path = "/node/{x}/{y}",
        responses(
            (status = 200, description = "Get the node info.", body = NodeData, example = json!(NodeData::random())),
        )
    )]
    pub(crate) async fn get_json(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Json<NodeData>> {
        Ok(Json(world.detect_node(NodeID(x, y)).await?))
    }

    #[utoipa::path(
        get,
        path = "/node/bytes/{x}/{y}",
        responses(
            (status = 200, description = "Get the node info in pure bytes (1024 byte length) format.", content_type = "application/octet-stream "),
        )
    )]
    pub(crate) async fn get_bytes(
        State(world): State<Arc<World>>,
        Path((x, y)): Path<(i16, i16)>,
    ) -> Result<Bytes> {
        Ok(Bytes::from(
            world.detect_node(NodeID(x, y)).await?.0.to_vec(),
        ))
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        soul::register,
        soul::get,
        guest::contain,
        guest::get,
        guest::walk,
        guest::harvest,
        guest::heat,
        node::get_json,
        node::get_bytes,
    ),
    components(
        schemas(
            soul::RegisterInfo,
            guest::WalkCommand,
            guest::HarvestCommand,
            guest::HeatCommand,
            crate::soul::Soul,
            crate::guest::GID,
            crate::guest::Guest,
            crate::node::NodeID,
            crate::node::NodeData,
        )
    ),
    tags(
        (name = "entropy", description = "Entropy game HTTP api.")
    )
)]
pub struct Doc;
