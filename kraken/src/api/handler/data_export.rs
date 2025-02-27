//! This module holds the data export of a workspace
//!
//! Data can be exported by an oauth application that was registered by an admin and has
//! access to a workspace granted by an user.

use std::collections::HashMap;

use actix_web::get;
use actix_web::web::{Data, Json, Path};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use ipnetwork::IpNetwork;
use rorm::prelude::*;
use rorm::{and, query, Database};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::extractors::BearerToken;
use crate::api::handler::{ApiError, ApiResult, PathUuid};
use crate::models::{
    Domain, DomainCertainty, DomainDomainRelation, DomainGlobalTag, DomainHostRelation,
    DomainWorkspaceTag, Host, HostCertainty, HostGlobalTag, HostWorkspaceTag, OsType, Port,
    PortCertainty, PortGlobalTag, PortProtocol, PortWorkspaceTag, Service, ServiceCertainty,
    ServiceGlobalTag, ServiceWorkspaceTag, WorkspaceAccessToken,
};

/// The aggregated results of a workspace
#[derive(Serialize, ToSchema)]
pub struct AggregatedWorkspace {
    /// The hosts found by this workspace
    pub hosts: HashMap<Uuid, AggregatedHost>,

    /// The ports found by this workspace
    pub ports: HashMap<Uuid, AggregatedPort>,

    /// The services found by this workspace
    pub services: HashMap<Uuid, AggregatedService>,

    /// The domains found by this workspace
    pub domains: HashMap<Uuid, AggregatedDomain>,

    /// All m2m relations which are not inlined
    pub relations: HashMap<Uuid, AggregatedRelation>,
}

/// A representation of an host.
#[derive(Serialize, ToSchema)]
pub struct AggregatedHost {
    /// The host's uuid
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    #[schema(value_type = String)]
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// The certainty of the host
    pub certainty: HostCertainty,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: Vec<Uuid>,

    /// The services of a host
    pub services: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainHost`]
    pub domains: Vec<Uuid>,

    /// A comment to the host
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this host was encountered
    pub created_at: DateTime<Utc>,
}

/// An open port on a host
#[derive(Serialize, ToSchema)]
pub struct AggregatedPort {
    /// The port's uuid
    pub uuid: Uuid,

    /// Port number
    pub port: u16,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The host this service is attached to
    pub host: Uuid,

    /// The services that link to this port
    pub services: Vec<Uuid>,

    /// The certainty of the port
    pub certainty: PortCertainty,

    /// A comment to the port
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this port was encountered
    pub created_at: DateTime<Utc>,
}

/// A detected service on a host
#[derive(Serialize, ToSchema)]
pub struct AggregatedService {
    /// The service's uuid
    pub uuid: Uuid,

    /// Name of the service
    pub name: String,

    /// Optional version of the service
    pub version: Option<String>,

    /// The host this service is attached to
    pub host: Uuid,

    /// The port this service is attached to
    pub port: Option<Uuid>,

    /// A comment to the service
    pub comment: String,

    /// The certainty the service was detected
    pub certainty: ServiceCertainty,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this service was encountered
    pub created_at: DateTime<Utc>,
}

/// A domain
#[derive(Serialize, ToSchema)]
pub struct AggregatedDomain {
    /// The domain's uuid
    pub uuid: Uuid,

    /// The domain that was found
    pub domain: String,

    /// Uuids to [`AggregatedRelation::DomainHost`]
    pub hosts: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `destination`
    pub sources: Vec<Uuid>,

    /// Uuids to [`AggregatedRelation::DomainDomain`] where this domain is the `source`
    pub destinations: Vec<Uuid>,

    /// The certainty of the domain
    pub certainty: DomainCertainty,

    /// A comment to the domain
    pub comment: String,

    /// Set of global and local tags
    #[serde(flatten)]
    pub tags: AggregatedTags,

    /// The first time this domain was encountered
    pub created_at: DateTime<Utc>,
}

/// Set of global and local tags
#[derive(Serialize, ToSchema, Default)]
pub struct AggregatedTags {
    /// Global tags
    global_tags: Vec<String>,

    /// Tags which are local to the workspace
    local_tags: Vec<String>,
}

/// An m2m relation
#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub enum AggregatedRelation {
    /// A DNS relation between two domains
    DomainDomain {
        /// The source domain pointing to the other domain
        source: Uuid,

        /// The destination domain which is pointed to by the other domain
        destination: Uuid,
    },
    /// A DNS relation between a domain and a host
    DomainHost {
        /// The domain resolving to a host
        domain: Uuid,

        /// The host resolved to by a domain
        host: Uuid,

        /// Does this relation exist directly as a dns record or is it the result of a chain of `CNAME`s?
        ///
        /// If this flag is set to `true`, the domain directly points to the host via an `A` or `AAAA` record.
        /// If it is `false`, the domain redirects to another via `CNAME` which eventually resolves to the host.
        is_direct: bool,
    },
}

#[utoipa::path(
    tag = "Data Export",
    context_path = "/api/v1/export",
    responses(
        (status = 200, description = "All hosts in the workspace", body = AggregatedWorkspace),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("bearer_token" = []))
)]
#[get("/workspace/{uuid}")]
pub(crate) async fn export_workspace(
    db: Data<Database>,
    path: Path<PathUuid>,
    token: BearerToken,
) -> ApiResult<Json<AggregatedWorkspace>> {
    let mut tx = db.start_transaction().await?;

    // Check access
    query!(&mut tx, (WorkspaceAccessToken::F.id,))
        .condition(and![
            WorkspaceAccessToken::F.token.equals(token.as_str()),
            WorkspaceAccessToken::F.workspace.equals(path.uuid),
            WorkspaceAccessToken::F.expires_at.greater_than(Utc::now()),
        ])
        .optional()
        .await?
        .ok_or(ApiError::MissingPrivileges)?;

    // Query all models without joins
    let mut hosts: HashMap<Uuid, AggregatedHost> = query!(&mut tx, Host)
        .condition(Host::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|host| (host.uuid, host.into()))
        .try_collect()
        .await?;
    let mut ports: HashMap<Uuid, AggregatedPort> = query!(&mut tx, Port)
        .condition(Port::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|port| (port.uuid, port.into()))
        .try_collect()
        .await?;
    let mut services: HashMap<Uuid, AggregatedService> = query!(&mut tx, Service)
        .condition(Service::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|service| (service.uuid, service.into()))
        .try_collect()
        .await?;
    let mut domains: HashMap<Uuid, AggregatedDomain> = query!(&mut tx, Domain)
        .condition(Domain::F.workspace.equals(path.uuid))
        .stream()
        .map_ok(|domain| (domain.uuid, domain.into()))
        .try_collect()
        .await?;
    let mut relations = HashMap::new();

    query!(&mut tx, DomainDomainRelation)
        .condition(DomainDomainRelation::F.workspace.equals(path.uuid))
        .stream()
        .try_for_each(|x| {
            relations.insert(
                x.uuid,
                AggregatedRelation::DomainDomain {
                    source: *x.source.key(),
                    destination: *x.destination.key(),
                },
            );
            if let Some(domain) = domains.get_mut(x.source.key()) {
                domain.destinations.push(x.uuid);
            }
            if let Some(domain) = domains.get_mut(x.destination.key()) {
                domain.sources.push(x.uuid);
            }
            async { Ok(()) }
        })
        .await?;

    query!(&mut tx, DomainHostRelation)
        .condition(DomainHostRelation::F.workspace.equals(path.uuid))
        .stream()
        .try_for_each(|x| {
            relations.insert(
                x.uuid,
                AggregatedRelation::DomainHost {
                    domain: *x.domain.key(),
                    host: *x.host.key(),
                    is_direct: x.is_direct,
                },
            );
            if let Some(host) = hosts.get_mut(x.host.key()) {
                host.domains.push(x.uuid);
            }
            if let Some(domain) = domains.get_mut(x.domain.key()) {
                domain.hosts.push(x.uuid);
            }
            async { Ok(()) }
        })
        .await?;

    // Resolve BackRefs manually
    for service in services.values() {
        if let Some(host) = hosts.get_mut(&service.host) {
            host.services.push(service.uuid);
        }
        if let Some(port) = service.port.as_ref() {
            if let Some(port) = ports.get_mut(port) {
                port.services.push(service.uuid);
            }
        }
    }
    for port in ports.values() {
        if let Some(host) = hosts.get_mut(&port.uuid) {
            host.ports.push(port.uuid);
        }
    }

    // Query all tags
    macro_rules! query_tags {
        ($owner:ident, $owner_set:ident, $GlobalTag:ident, $WorkspaceTag:ident) => {
            let mut stream = query!(
                &mut tx,
                ($GlobalTag::F.$owner.uuid, $GlobalTag::F.global_tag.name)
            )
            .condition($GlobalTag::F.$owner.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(&owner_uuid) {
                    owner.tags.global_tags.push(name);
                }
            }
            drop(stream);
            let mut stream = query!(
                &mut tx,
                ($WorkspaceTag::F.$owner, $WorkspaceTag::F.workspace_tag.name)
            )
            .condition($WorkspaceTag::F.workspace_tag.workspace.equals(path.uuid))
            .stream();
            while let Some((owner_uuid, name)) = stream.try_next().await? {
                if let Some(owner) = $owner_set.get_mut(owner_uuid.key()) {
                    owner.tags.local_tags.push(name);
                }
            }
            drop(stream);
        };
    }
    query_tags!(host, hosts, HostGlobalTag, HostWorkspaceTag);
    query_tags!(port, ports, PortGlobalTag, PortWorkspaceTag);
    query_tags!(service, services, ServiceGlobalTag, ServiceWorkspaceTag);
    query_tags!(domain, domains, DomainGlobalTag, DomainWorkspaceTag);

    tx.commit().await?;
    Ok(Json(AggregatedWorkspace {
        hosts,
        ports,
        services,
        domains,
        relations,
    }))
}

impl From<Host> for AggregatedHost {
    fn from(value: Host) -> Self {
        let Host {
            uuid,
            ip_addr,
            os_type,
            response_time,
            ports: _,
            services: _,
            domains: _,
            comment,
            workspace: _,
            workspace_tags: _,
            global_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            ip_addr,
            os_type,
            response_time,
            certainty,
            ports: Vec::new(),
            services: Vec::new(),
            domains: Vec::new(),
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
impl From<Port> for AggregatedPort {
    fn from(value: Port) -> Self {
        let Port {
            uuid,
            port,
            protocol,
            host,
            services: _,
            comment,
            workspace: _,
            global_tags: _,
            workspace_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            port: u16::from_ne_bytes(port.to_ne_bytes()),
            protocol,
            host: *host.key(),
            services: Vec::new(),
            certainty,
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
impl From<Service> for AggregatedService {
    fn from(value: Service) -> Self {
        let Service {
            uuid,
            name,
            version,
            host,
            port,
            comment,
            certainty,
            workspace: _,
            workspace_tags: _,
            global_tags: _,
            created_at,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            name,
            version,
            host: *host.key(),
            port: port.map(|port| *port.key()),
            comment,
            certainty,
            tags: Default::default(),
            created_at,
        }
    }
}
impl From<Domain> for AggregatedDomain {
    fn from(value: Domain) -> Self {
        let Domain {
            uuid,
            domain,
            comment,
            hosts: _,
            sources: _,
            destinations: _,
            workspace: _,
            workspace_tags: _,
            global_tags: _,
            created_at,
            certainty,
        } = value;
        // DON'T just ignore new fields with `: _`
        // Make sure you export the field in some other way!

        Self {
            uuid,
            domain,
            hosts: Vec::new(),
            sources: Vec::new(),
            destinations: Vec::new(),
            certainty,
            comment,
            tags: Default::default(),
            created_at,
        }
    }
}
