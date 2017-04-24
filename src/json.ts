import { Dict } from './interfaces/core';

/** JSON data, as returned by `JSON.parse()`. */
export type JSONValue = null | boolean | number | string | JSONObject | JSONArray;

/** JSON object values. */
export interface JSONObject extends Dict<JSONValue> {}

/** JSON array values. */
export interface JSONArray extends Array<JSONValue> {}

/** Tests a JSON value to see if it is a JSON object (aka dict or record). */
export function isJSONObject(x: JSONValue): x is JSONObject {
    return !!x && typeof x === 'object' && !Array.isArray(x);
}

/** Tests a JSON value to see if it is a JSON array. */
export function isJSONArray(x: JSONValue): x is JSONArray {
    return Array.isArray(x);
}
