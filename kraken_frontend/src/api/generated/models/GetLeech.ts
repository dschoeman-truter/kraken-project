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
 * @interface GetLeech
 */
export interface GetLeech {
    /**
     * 
     * @type {number}
     * @memberof GetLeech
     */
    id: number;
    /**
     * 
     * @type {string}
     * @memberof GetLeech
     */
    name: string;
    /**
     * 
     * @type {string}
     * @memberof GetLeech
     */
    address: string;
}

export function GetLeechFromJSON(json: any): GetLeech {
    return GetLeechFromJSONTyped(json, false);
}

export function GetLeechFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetLeech {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'id': json['id'],
        'name': json['name'],
        'address': json['address'],
    };
}

export function GetLeechToJSON(value?: GetLeech | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'id': value.id,
        'name': value.name,
        'address': value.address,
    };
}


