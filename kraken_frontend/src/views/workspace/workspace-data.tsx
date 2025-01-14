import React from "react";
import "../../styling/workspace-data.css";
import { StatelessWorkspaceTable, useTable } from "./components/workspace-table";
import { Api } from "../../api/api";
import Tag from "../../components/tag";
import { SimpleTag } from "../../api/generated";

const TABS = { domains: "Domains", hosts: "Hosts", ports: "Ports", services: "Services", other: "Other" };

type WorkspaceDataProps = {
    /** Workspace uuid */
    workspace: string;
};

export default function WorkspaceData(props: WorkspaceDataProps) {
    const { workspace } = props;

    const [tab, setTab] = React.useState<keyof typeof TABS>("hosts");

    const { items: domains, ...domainsTable } = useTable(
        (limit, offset) => Api.workspaces.domains.all(workspace, limit, offset),
        [workspace],
    );
    const { items: hosts, ...hostsTable } = useTable(
        (limit, offset) => Api.workspaces.hosts.all(workspace, limit, offset),
        [workspace],
    );
    const { items: ports, ...portsTable } = useTable(
        (limit, offset) => Api.workspaces.ports.all(workspace, limit, offset),
        [workspace],
    );
    const { items: services, ...servicesTable } = useTable(
        (limit, offset) => Api.workspaces.services.all(workspace, limit, offset),
        [workspace],
    );

    const tableElement = (() => {
        switch (tab) {
            case "domains":
                return (
                    <StatelessWorkspaceTable {...domainsTable}>
                        <div className={"workspace-data-table-header"}>
                            <span>Name</span>
                            <span>Tags</span>
                            <span>Comment</span>
                        </div>
                        {domains.map((domain) => (
                            <div className={"workspace-data-table-row"}>
                                <span>{domain.domain}</span>
                                <TagList tags={domain.tags} />
                                <span>{domain.comment}</span>
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "hosts":
                return (
                    <StatelessWorkspaceTable {...hostsTable}>
                        <div className={"workspace-data-table-header"}>
                            <span>IP</span>
                            <span>Tags</span>
                            <span>Comment</span>
                        </div>
                        {hosts.map((host) => (
                            <div className={"workspace-data-table-row"}>
                                <span>{host.ipAddr}</span>
                                <TagList tags={host.tags} />
                                <span>{host.comment}</span>
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "ports":
                return (
                    <StatelessWorkspaceTable {...portsTable}>
                        <div className={"workspace-data-table-header"}>
                            <span>Port</span>
                            <span>Host</span>
                            <span>Tags</span>
                            <span>Comment</span>
                        </div>
                        {ports.map((port) => (
                            <div className={"workspace-data-table-row"}>
                                <span>{port.port}</span>
                                <span>{port.host.ipAddr}</span>
                                <TagList tags={port.tags} />
                                <span>{port.comment}</span>
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            case "services":
                return (
                    <StatelessWorkspaceTable {...servicesTable}>
                        <div className={"workspace-data-table-header"}>
                            <span>Name</span>
                            <span>Host</span>
                            <span>Port</span>
                            <span>Tags</span>
                            <span>Comment</span>
                        </div>
                        {services.map((service) => (
                            <div className={"workspace-data-table-row"}>
                                <span>{service.name}</span>
                                <span>{service.host.ipAddr}</span>
                                <span>{service.port?.port}</span>
                                <TagList tags={service.tags} />
                                <span>{service.comment}</span>
                            </div>
                        ))}
                    </StatelessWorkspaceTable>
                );
            default:
                return "Unimplemented";
        }
    })();
    return (
        <div className={"workspace-data-container"}>
            <div className={"workspace-data-selector"}>
                {Object.entries(TABS).map(([key, displayName]) => (
                    <div
                        className={"pane" + (tab !== key ? "" : " workspace-data-selected-tab")}
                        onClick={() => setTab(key as keyof typeof TABS)}
                    >
                        <h3 className={"heading"}>{displayName}</h3>
                    </div>
                ))}
            </div>
            {tableElement}
            <div className={"workspace-data-details pane"}>
                <h2 className={"heading"}>Details</h2>
            </div>
        </div>
    );
}

function TagList(props: { tags: Array<SimpleTag> }) {
    return (
        <div className={"tag-list"}>
            {props.tags.map((tag) => (
                <Tag key={tag.uuid} {...tag} />
            ))}
        </div>
    );
}
