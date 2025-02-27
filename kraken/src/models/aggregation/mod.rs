use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use rorm::prelude::{BackRef, ForeignModel};
use rorm::{field, DbEnum, Model};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub(crate) use crate::models::aggregation::operations::*;
use crate::models::{GlobalTag, Workspace, WorkspaceTag};

mod operations;

/// A representation of an OS type
#[derive(DbEnum, Copy, Clone, Debug, ToSchema, Serialize)]
pub enum OsType {
    /// The OS type is currently unknown
    Unknown,
    /// Linux based OS
    Linux,
    /// Windows based OS
    Windows,
    /// Apple based OS
    Apple,
    /// Android based OS
    Android,
    /// FreeBSD based OS
    FreeBSD,
}

/// The certainty of a host
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum HostCertainty {
    /// 3rd party historical data
    Historical,
    /// 3rd party data
    SupposedTo,
    /// The host has responded either by HostAlive, Port or Service Detection or something similar
    Verified,
}

/// A representation of an host.
///
/// Will be collected from all results that yield IP addresses
#[derive(Model)]
pub struct Host {
    /// The primary key of a host
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The IP address of the host.
    ///
    /// If the host has multiple addresses, create a [Host] for each and link them.
    pub ip_addr: IpNetwork,

    /// The type of OS of this host
    pub os_type: OsType,

    /// Response time in ms
    pub response_time: Option<i32>,

    /// The ports of a host
    pub ports: BackRef<field!(Port::F.host)>,

    /// The services of a host
    pub services: BackRef<field!(Service::F.host)>,

    /// The domains of a host
    pub domains: BackRef<field!(DomainHostRelation::F.host)>,

    /// A comment to the host
    #[rorm(max_length = 255)]
    pub comment: String,

    /// The certainty of this host
    pub certainty: HostCertainty,

    /// Workspace tags of the host
    pub workspace_tags: BackRef<field!(HostWorkspaceTag::F.host)>,

    /// Global tags of the host
    pub global_tags: BackRef<field!(HostGlobalTag::F.host)>,

    /// A reference to the workspace this host is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Host]
#[derive(Model)]
pub struct HostGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The host this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: ForeignModel<Host>,
}

/// M2M relation between [WorkspaceTag] and [Host]
#[derive(Model)]
pub struct HostWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The host this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub host: ForeignModel<Host>,
}

/// The certainty a service is detected
#[derive(Debug, Copy, Clone, ToSchema, Deserialize, Serialize, DbEnum, Eq, PartialEq)]
pub enum ServiceCertainty {
    /// 3rd party historical data
    Historical,
    /// 3rd party data
    SupposedTo,
    /// May be a certain service
    MaybeVerified,
    /// Service is definitely correct
    DefinitelyVerified,
}

/// A detected service on a host
#[derive(Model)]
pub struct Service {
    /// Primary key of a service
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Name of the service
    #[rorm(max_length = 255)]
    pub name: String,

    /// Optional version of the service
    #[rorm(max_length = 255)]
    pub version: Option<String>,

    /// The certainty the service is detected correct
    pub certainty: ServiceCertainty,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The port this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub port: Option<ForeignModel<Port>>,

    /// A comment to the service
    #[rorm(max_length = 255)]
    pub comment: String,

    /// Workspace tags of the service
    pub workspace_tags: BackRef<field!(ServiceWorkspaceTag::F.service)>,

    /// Global tags of the service
    pub global_tags: BackRef<field!(ServiceGlobalTag::F.service)>,

    /// A reference to the workspace this service is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Service]
#[derive(Model)]
pub struct ServiceGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The service this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub service: ForeignModel<Service>,
}

/// M2M relation between [WorkspaceTag] and [Service]
#[derive(Model)]
pub struct ServiceWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The service this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub service: ForeignModel<Service>,
}

/// A protocol of a port
#[derive(DbEnum, ToSchema, Debug, Copy, Clone, Serialize)]
pub enum PortProtocol {
    /// Unknown protocol
    Unknown,
    /// tcp
    Tcp,
    /// udp
    Udp,
    /// sctp
    Sctp,
}

/// The certainty states of a port
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum PortCertainty {
    /// 3rd party historical data
    Historical,
    /// 3rd party data
    SupposedTo,
    /// The host has responded either by HostAlive, Port or Service Detection or something similar
    Verified,
}

/// A port
#[derive(Model)]
pub struct Port {
    /// Primary key of a port
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// Port number
    ///
    /// Reinterpret as u16 with to_ne_bytes and from_ne_bytes
    pub port: i16,

    /// Port protocol
    pub protocol: PortProtocol,

    /// The certainty of this port
    pub certainty: PortCertainty,

    /// The host this service is attached to
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub host: ForeignModel<Host>,

    /// The services that link to this port
    pub services: BackRef<field!(Service::F.port)>,

    /// A comment to the port
    #[rorm(max_length = 255)]
    pub comment: String,

    /// Workspace tags of the port
    pub workspace_tags: BackRef<field!(PortWorkspaceTag::F.port)>,

    /// Global tags of the port
    pub global_tags: BackRef<field!(PortGlobalTag::F.port)>,

    /// A reference to the workspace this port is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between [GlobalTag] and [Port]
#[derive(Model)]
pub struct PortGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The port this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: ForeignModel<Port>,
}

/// M2M relation between [WorkspaceTag] and [Port]
#[derive(Model)]
pub struct PortWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The port this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub port: ForeignModel<Port>,
}

/// The certainty of a domain
#[derive(DbEnum, Copy, Clone, Deserialize, Serialize, ToSchema, Debug)]
pub enum DomainCertainty {
    /// The domain was not found through DNS
    Unverified,
    /// Domain was verified through DNS
    Verified,
}

/// A domain
#[derive(Model)]
pub struct Domain {
    /// The primary key of a domain
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The domain that was found
    #[rorm(max_length = 255)]
    pub domain: String,

    /// The certainty of this domain entry
    pub certainty: DomainCertainty,

    /// A comment to the domain
    #[rorm(max_length = 255)]
    pub comment: String,

    /// Domains resolving to this host
    pub hosts: BackRef<field!(DomainHostRelation::F.domain)>,

    /// Domains pointing to this one
    pub sources: BackRef<field!(DomainDomainRelation::F.destination)>,

    /// Domains, this one resolves to
    pub destinations: BackRef<field!(DomainDomainRelation::F.source)>,

    /// Workspace tags of the domain
    pub workspace_tags: BackRef<field!(DomainWorkspaceTag::F.domain)>,

    /// Global tags of the domain
    pub global_tags: BackRef<field!(DomainGlobalTag::F.domain)>,

    /// A reference to the workspace this domain is referencing
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,

    /// The point in time, this entry was created
    #[rorm(auto_create_time)]
    pub created_at: DateTime<Utc>,
}

/// M2M relation between two [domains](Domain)
#[derive(Model)]
pub struct DomainDomainRelation {
    /// The primary key of this relation
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The source address
    pub source: ForeignModel<Domain>,

    /// The destination address
    pub destination: ForeignModel<Domain>,

    /// A reference to the workspace for faster querying
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

/// M2M relation between a [Domain] and a [Host]
#[derive(Model)]
pub struct DomainHostRelation {
    /// The primary key of this relation
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The source domain
    pub domain: ForeignModel<Domain>,

    /// The destination host
    pub host: ForeignModel<Host>,

    /// Does this relation exist directly as a dns record or is it the result of a chain of `CNAME`s?
    ///
    /// If this flag is set to `true`, the domain directly points to the host via an `A` or `AAAA` record.
    /// If it is `false`, the domain redirects to another via `CNAME` which eventually resolves to the host.
    pub is_direct: bool,

    /// A reference to the workspace for faster querying
    #[rorm(on_delete = "Cascade", on_update = "Cascade")]
    pub workspace: ForeignModel<Workspace>,
}

/* This enum won't be actually used, but stays for now as reminder and collection of which relations will need implementations

/// The type of a relation
#[derive(DbEnum)]
pub enum RelationType {
    /// Relation to an IPv4 address
    A,
    /// Relation to an IPv6 address
    AAAA,
    /// Relation to another domain
    CNAME,
    /// Relation from an SPF record
    SPF,
    /// Relation from an SRV record
    SRV,
    /// Relation from an TXT record
    TXT,
    /// Relation from an NS record
    NS,
    /// Relation from an SOA record
    SOA,
    /// Relation from an MX record
    MX,
    /// Relation from an PTR record
    PTR,
    /// Relation from an TLSA record
    TLSA,
    /// Relation from an CAA record
    CAA,
    /// Relation from an DMARC record
    DMARC,
}
*/

/// M2M relation between [GlobalTag] and [Domain]
#[derive(Model)]
pub struct DomainGlobalTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The global tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub global_tag: ForeignModel<GlobalTag>,

    /// The domain this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: ForeignModel<Domain>,
}

/// M2M relation between [WorkspaceTag] and [Domain]
#[derive(Model)]
pub struct DomainWorkspaceTag {
    /// Primary key of the entry
    #[rorm(primary_key)]
    pub uuid: Uuid,

    /// The workspace tag this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub workspace_tag: ForeignModel<WorkspaceTag>,

    /// The domain this entry links to
    #[rorm(on_update = "Cascade", on_delete = "Cascade")]
    pub domain: ForeignModel<Domain>,
}
