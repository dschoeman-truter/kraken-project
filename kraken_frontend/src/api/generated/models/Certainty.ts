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


/**
 * The certainty a service is detected
 * @export
 */
export const Certainty = {
    Unknown: 'Unknown',
    Maybe: 'Maybe',
    Definitely: 'Definitely'
} as const;
export type Certainty = typeof Certainty[keyof typeof Certainty];


export function CertaintyFromJSON(json: any): Certainty {
    return CertaintyFromJSONTyped(json, false);
}

export function CertaintyFromJSONTyped(json: any, ignoreDiscriminator: boolean): Certainty {
    return json as Certainty;
}

export function CertaintyToJSON(value?: Certainty | null): any {
    return value as any;
}

