//! Everything regarding workspace management is located in this module

use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};
use chrono::{DateTime, Utc};
use log::{debug, warn};
use rorm::db::transaction::Transaction;
use rorm::prelude::ForeignModelByField;
use rorm::{and, query, update, Database, FieldAccess, Model};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::attacks::SimpleAttack;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspace_invitations::{
    FullWorkspaceInvitation, WorkspaceInvitationList,
};
use crate::api::handler::{de_optional, query_user, ApiError, ApiResult, PathUuid, UuidResponse};
use crate::chan::{WsManagerChan, WsManagerMessage, WsMessage};
use crate::models::{
    Attack, User, UserPermission, Workspace, WorkspaceInvitation, WorkspaceMember,
};

/// The request to create a new workspace
#[derive(Deserialize, ToSchema)]
pub struct CreateWorkspaceRequest {
    #[schema(example = "secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is super secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
}

/// Create a new workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was created", body = UuidResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    request_body = CreateWorkspaceRequest,
    security(("api_key" = []))
)]
#[post("/workspaces")]
pub async fn create_workspace(
    req: Json<CreateWorkspaceRequest>,
    db: Data<Database>,
    session: SessionUser,
) -> ApiResult<Json<UuidResponse>> {
    let req = req.into_inner();

    let uuid = Workspace::insert(db.as_ref(), req.name, req.description, session.0).await?;

    Ok(Json(UuidResponse { uuid }))
}

/// Delete a workspace by its id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was deleted"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[delete("/workspaces/{uuid}")]
pub async fn delete_workspace(
    req: Path<PathUuid>,
    session: Session,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let mut tx = db.start_transaction().await?;

    let executing_user = query_user(&mut tx, &session).await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(req.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if executing_user.permission == UserPermission::Admin
        || *workspace.owner.key() == executing_user.uuid
    {
        debug!(
            "Workspace {} got deleted by {}",
            workspace.uuid, executing_user.username
        );

        rorm::delete!(&mut tx, Workspace).single(&workspace).await?;
    } else {
        debug!(
            "User {} does not has the privileges to delete the workspace {}",
            executing_user.username, workspace.uuid
        );

        return Err(ApiError::MissingPrivileges);
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// A simple version of a workspace
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct SimpleWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: SimpleUser,
    pub(crate) created_at: DateTime<Utc>,
}

/// A full version of a workspace
#[derive(Serialize, ToSchema)]
pub struct FullWorkspace {
    pub(crate) uuid: Uuid,
    #[schema(example = "ultra-secure-workspace")]
    pub(crate) name: String,
    #[schema(example = "This workspace is ultra secure and should not be looked at!!")]
    pub(crate) description: Option<String>,
    pub(crate) owner: SimpleUser,
    pub(crate) attacks: Vec<SimpleAttack>,
    pub(crate) members: Vec<SimpleUser>,
    pub(crate) created_at: DateTime<Utc>,
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns the workspace", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace(
    req: Path<PathUuid>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<FullWorkspace>> {
    let user_uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let mut tx = db.start_transaction().await?;

    let workspace = if Workspace::is_user_member_or_owner(&mut tx, req.uuid, user_uuid).await? {
        get_workspace_unchecked(req.uuid, &mut tx).await
    } else {
        Err(ApiError::MissingPrivileges)
    };

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// The response to retrieve all workspaces
#[derive(Serialize, ToSchema)]
pub struct GetAllWorkspacesResponse {
    pub(crate) workspaces: Vec<SimpleWorkspace>,
}

/// Retrieve all workspaces owned by executing user
///
/// For administration access, look at the `/admin/workspaces` endpoint.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all workspaces owned by the executing user", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces(
    db: Data<Database>,
    session: Session,
) -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = db.start_transaction().await?;

    let owner = query_user(&mut tx, &session).await?;

    let workspaces = query!(&mut tx, Workspace)
        .condition(Workspace::F.owner.equals(owner.uuid))
        .all()
        .await?;

    tx.commit().await?;

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| SimpleWorkspace {
                uuid: w.uuid,
                name: w.name,
                description: w.description,
                owner: SimpleUser {
                    uuid: owner.uuid,
                    username: owner.username.clone(),
                    display_name: owner.display_name.clone(),
                },
                created_at: w.created_at,
            })
            .collect(),
    }))
}

/// The request type to update a workspace
///
/// All parameter are optional, but at least one of them must be specified
#[derive(Deserialize, ToSchema)]
pub struct UpdateWorkspaceRequest {
    #[schema(example = "Workspace for work")]
    name: Option<String>,
    #[schema(example = "This workspace is for work and for work only!")]
    #[serde(deserialize_with = "de_optional")]
    description: Option<Option<String>>,
}

/// Updates a workspace by its id
///
/// All parameter are optional, but at least one of them must be specified.
///
/// `name` must not be empty.
///
/// You can set `description` to null to remove the description from the database.
/// If you leave the parameter out, the description will remain unchanged.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace got updated"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = UpdateWorkspaceRequest,
    security(("api_key" = []))
)]
#[put("/workspaces/{uuid}")]
pub async fn update_workspace(
    path: Path<PathUuid>,
    req: Json<UpdateWorkspaceRequest>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let uuid: Uuid = session.get("uuid")?.ok_or(ApiError::SessionCorrupt)?;

    let req = req.into_inner();

    let mut tx = db.start_transaction().await?;

    let w = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(path.uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    if *w.owner.key() != uuid {
        return Err(ApiError::MissingPrivileges);
    }

    if let Some(name) = &req.name {
        if name.is_empty() {
            return Err(ApiError::InvalidName);
        }
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(w.uuid))
        .begin_dyn_set()
        .set_if(Workspace::F.name, req.name)
        .set_if(Workspace::F.description, req.description)
        .finish_dyn_set()
        .map_err(|_| ApiError::EmptyJson)?
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// The request to transfer a workspace to another account
#[derive(Debug, ToSchema, Deserialize)]
pub struct TransferWorkspaceRequest {
    /// The uuid of the user that should receive the workspace
    pub user: Uuid,
}

/// Transfer ownership to another account
///
/// You will loose access to the workspace.
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Workspace was transferred"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/transfer")]
pub async fn transfer_ownership(
    req: Json<TransferWorkspaceRequest>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
    db: Data<Database>,
) -> ApiResult<HttpResponse> {
    let new_owner_uuid = req.into_inner().user;
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = db.start_transaction().await?;

    let Some(workspace) = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
    else {
        return Err(ApiError::MissingPrivileges);
    };

    if *workspace.owner.key() != user_uuid {
        return Err(ApiError::MissingPrivileges);
    }

    update!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .set(Workspace::F.owner, ForeignModelByField::Key(new_owner_uuid))
        .exec()
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// The request to invite a user to the workspace
#[derive(Deserialize, Debug, ToSchema)]
pub struct InviteToWorkspace {
    /// The user to invite
    pub user: Uuid,
}

/// Invite a user to the workspace
///
/// This action can only be invoked by the owner of a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The user was invited."),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    request_body = InviteToWorkspace,
    security(("api_key" = []))
)]
#[post("/workspaces/{uuid}/invitations")]
pub async fn create_invitation(
    req: Json<InviteToWorkspace>,
    path: Path<PathUuid>,
    ws_manager_chan: Data<WsManagerChan>,
    db: Data<Database>,
    session: Session,
) -> ApiResult<HttpResponse> {
    let InviteToWorkspace { user } = req.into_inner();
    let workspace = path.into_inner().uuid;

    let mut tx = db.start_transaction().await?;
    let session_user = query_user(&mut tx, &session).await?;

    WorkspaceInvitation::insert(db.as_ref(), workspace, session_user.uuid, user).await?;

    tx.commit().await?;

    if let Err(err) = ws_manager_chan
        .send(WsManagerMessage::Message(
            user,
            WsMessage::InvitationToWorkspace {
                from: SimpleUser {
                    uuid: session_user.uuid,
                    username: session_user.username,
                    display_name: session_user.display_name,
                },
                workspace_uuid: workspace,
            },
        ))
        .await
    {
        warn!("Could not send to ws manager chan: {err}")
    }

    Ok(HttpResponse::Ok().finish())
}

/// The url components of an invitation
#[derive(Deserialize, IntoParams)]
pub struct InviteUuid {
    /// The UUID of the workspace
    pub w_uuid: Uuid,
    /// The UUID of the invitation
    pub i_uuid: Uuid,
}

/// Retract an invitation to the workspace
///
/// This action can only be invoked by the owner of a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "The invitation was retracted."),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(InviteUuid),
    security(("api_key" = []))
)]
#[delete("/workspaces/{w_uuid}/invitations/{i_uuid}")]
pub async fn retract_invitation(
    path: Path<InviteUuid>,
    db: Data<Database>,
    SessionUser(session_user): SessionUser,
) -> ApiResult<HttpResponse> {
    let InviteUuid { w_uuid, i_uuid } = path.into_inner();

    let mut tx = db.start_transaction().await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(w_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidWorkspace)?;

    if *workspace.owner.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    query!(&mut tx, (WorkspaceInvitation::F.uuid,))
        .condition(and!(
            WorkspaceInvitation::F.uuid.equals(i_uuid),
            WorkspaceInvitation::F.workspace.equals(w_uuid)
        ))
        .optional()
        .await?
        .ok_or(ApiError::InvalidInvitation)?;

    rorm::delete!(&mut tx, WorkspaceInvitation)
        .condition(WorkspaceInvitation::F.uuid.equals(i_uuid))
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

/// Query all open invitations to a workspace
#[utoipa::path(
    tag = "Workspaces",
    context_path = "/api/v1",
    responses(
        (status = 200, description = "Returns all open invitations to the workspace.", body = WorkspaceInvitationList),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}/invitations")]
pub async fn get_all_workspace_invitations(
    path: Path<PathUuid>,
    db: Data<Database>,
    SessionUser(session_user): SessionUser,
) -> ApiResult<Json<WorkspaceInvitationList>> {
    let workspace_uuid = path.into_inner().uuid;

    let mut tx = db.start_transaction().await?;

    let workspace = query!(&mut tx, Workspace)
        .condition(Workspace::F.uuid.equals(workspace_uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidWorkspace)?;

    if *workspace.owner.key() != session_user {
        return Err(ApiError::MissingPrivileges);
    }

    let invitations = query!(
        &mut tx,
        (
            WorkspaceInvitation::F.uuid,
            WorkspaceInvitation::F.workspace as Workspace,
            WorkspaceInvitation::F.from as SimpleUser,
            WorkspaceInvitation::F.target as SimpleUser,
            WorkspaceInvitation::F.workspace.owner as SimpleUser
        )
    )
    .condition(WorkspaceInvitation::F.workspace.equals(workspace_uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(uuid, workspace, from, target, owner)| FullWorkspaceInvitation {
            uuid,
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                owner,
                name: workspace.name,
                description: workspace.description,
                created_at: workspace.created_at,
            },
            from,
            target,
        },
    )
    .collect();

    tx.commit().await?;

    Ok(Json(WorkspaceInvitationList { invitations }))
}

/// Retrieve a workspace by id
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns the workspace with the given id", body = FullWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/workspaces/{uuid}")]
pub async fn get_workspace_admin(
    req: Path<PathUuid>,
    db: Data<Database>,
) -> ApiResult<Json<FullWorkspace>> {
    let mut tx = db.start_transaction().await?;

    let workspace = get_workspace_unchecked(req.uuid, &mut tx).await;

    tx.commit().await?;

    Ok(Json(workspace?))
}

/// Retrieve all workspaces
#[utoipa::path(
    tag = "Admin Workspaces",
    context_path = "/api/v1/admin",
    responses(
        (status = 200, description = "Returns all workspaces", body = GetAllWorkspacesResponse),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    security(("api_key" = []))
)]
#[get("/workspaces")]
pub async fn get_all_workspaces_admin(
    db: Data<Database>,
) -> ApiResult<Json<GetAllWorkspacesResponse>> {
    let mut tx = db.start_transaction().await?;

    let workspaces = query!(
        &mut tx,
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.owner.uuid,
            Workspace::F.owner.username,
            Workspace::F.owner.display_name
        )
    )
    .all()
    .await?;

    tx.commit().await?;

    Ok(Json(GetAllWorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(
                |(uuid, name, description, created_at, by_uuid, username, display_name)| {
                    SimpleWorkspace {
                        uuid,
                        name,
                        description,
                        owner: SimpleUser {
                            uuid: by_uuid,
                            username,
                            display_name,
                        },
                        created_at,
                    }
                },
            )
            .collect(),
    }))
}

/// Get a [`FullWorkspace`] by its uuid without permission checks
async fn get_workspace_unchecked(uuid: Uuid, tx: &mut Transaction) -> ApiResult<FullWorkspace> {
    let workspace = query!(&mut *tx, Workspace)
        .condition(Workspace::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let owner = query!(&mut *tx, User)
        .condition(User::F.uuid.equals(*workspace.owner.key()))
        .one()
        .await?;

    let attacks = query!(
        &mut *tx,
        (
            Attack::F.uuid,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by.uuid,
            Attack::F.started_by.username,
            Attack::F.started_by.display_name,
        )
    )
    .condition(Attack::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(attack_uuid, attack_type, finished_at, created_at, by_uuid, username, display_name)| {
            SimpleAttack {
                uuid: attack_uuid,
                workspace_uuid: uuid,
                attack_type,
                started_from: SimpleUser {
                    uuid: by_uuid,
                    username,
                    display_name,
                },
                finished_at,
                created_at,
            }
        },
    )
    .collect();

    let members = query!(
        &mut *tx,
        (
            WorkspaceMember::F.member.uuid,
            WorkspaceMember::F.member.username,
            WorkspaceMember::F.member.display_name
        )
    )
    .condition(WorkspaceMember::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(|(uuid, username, display_name)| SimpleUser {
        uuid,
        username,
        display_name,
    })
    .collect();

    Ok(FullWorkspace {
        uuid: workspace.uuid,
        name: workspace.name,
        description: workspace.description,
        owner: SimpleUser {
            uuid: owner.uuid,
            username: owner.username,
            display_name: owner.display_name,
        },
        attacks,
        members,
        created_at: workspace.created_at,
    })
}
