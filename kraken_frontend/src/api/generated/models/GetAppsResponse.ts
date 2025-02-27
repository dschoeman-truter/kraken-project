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
import type { FullOauthClient } from './FullOauthClient';
import {
    FullOauthClientFromJSON,
    FullOauthClientFromJSONTyped,
    FullOauthClientToJSON,
} from './FullOauthClient';

/**
 * 
 * @export
 * @interface GetAppsResponse
 */
export interface GetAppsResponse {
    /**
     * 
     * @type {Array<FullOauthClient>}
     * @memberof GetAppsResponse
     */
    apps: Array<FullOauthClient>;
}

/**
 * Check if a given object implements the GetAppsResponse interface.
 */
export function instanceOfGetAppsResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "apps" in value;

    return isInstance;
}

export function GetAppsResponseFromJSON(json: any): GetAppsResponse {
    return GetAppsResponseFromJSONTyped(json, false);
}

export function GetAppsResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetAppsResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'apps': ((json['apps'] as Array<any>).map(FullOauthClientFromJSON)),
    };
}

export function GetAppsResponseToJSON(value?: GetAppsResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'apps': ((value.apps as Array<any>).map(FullOauthClientToJSON)),
    };
}

