/** An opaque type that matches all JS values, like `any` but with stricter type checking. */
type unknown = {} | null | undefined | void;

/** Generic type of dynamic dictionary objects. */
export interface Dict<T> {
  [key: string]: T;
}
