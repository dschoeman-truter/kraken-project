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
/**
 * 
 * @export
 * @interface CreateLeechRequest
 */
export interface CreateLeechRequest {
    /**
     * 
     * @type {string}
     * @memberof CreateLeechRequest
     */
    name: string;
    /**
     * 
     * @type {string}
     * @memberof CreateLeechRequest
     */
    address: string;
    /**
     * 
     * @type {string}
     * @memberof CreateLeechRequest
     */
    description?: string;
}

export function CreateLeechRequestFromJSON(json: any): CreateLeechRequest {
    return CreateLeechRequestFromJSONTyped(json, false);
}

export function CreateLeechRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): CreateLeechRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'name': json['name'],
        'address': json['address'],
        'description': !exists(json, 'description') ? undefined : json['description'],
    };
}

export function CreateLeechRequestToJSON(value?: CreateLeechRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'name': value.name,
        'address': value.address,
        'description': value.description,
    };
}


