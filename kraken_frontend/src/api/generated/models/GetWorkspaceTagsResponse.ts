/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: git@omikron.dev
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { FullWorkspaceTag } from './FullWorkspaceTag';
import {
    FullWorkspaceTagFromJSON,
    FullWorkspaceTagFromJSONTyped,
    FullWorkspaceTagToJSON,
} from './FullWorkspaceTag';

/**
 * The response to a request to retrieve all workspace tags
 * @export
 * @interface GetWorkspaceTagsResponse
 */
export interface GetWorkspaceTagsResponse {
    /**
     * 
     * @type {Array<FullWorkspaceTag>}
     * @memberof GetWorkspaceTagsResponse
     */
    workspaceTags: Array<FullWorkspaceTag>;
}

/**
 * Check if a given object implements the GetWorkspaceTagsResponse interface.
 */
export function instanceOfGetWorkspaceTagsResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "workspaceTags" in value;

    return isInstance;
}

export function GetWorkspaceTagsResponseFromJSON(json: any): GetWorkspaceTagsResponse {
    return GetWorkspaceTagsResponseFromJSONTyped(json, false);
}

export function GetWorkspaceTagsResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetWorkspaceTagsResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'workspaceTags': ((json['workspace_tags'] as Array<any>).map(FullWorkspaceTagFromJSON)),
    };
}

export function GetWorkspaceTagsResponseToJSON(value?: GetWorkspaceTagsResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'workspace_tags': ((value.workspaceTags as Array<any>).map(FullWorkspaceTagToJSON)),
    };
}

