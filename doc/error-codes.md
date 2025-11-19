# Neon Error Codes

Neon compiler errors include a short code in brackets. The table below lists each code and the associated message.

| Code  | Message |
|-------|---------|
| N0001 | `neon::export` can only be applied to functions, consts, and statics. |
| N0002 | Expected `&mut Cx` instead of a `Channel` reference. |
| N0003 | Context must be a `&mut` reference. |
| N0004 | Expected `&mut Cx` instead of `Channel`. |
| N0005 | Must be a `&mut` reference. |
| N0006 | Expected an owned `Channel` instead of a reference. |
| N0007 | Expected an owned `Channel` instead of a context reference. |
| N0008 | Context is not available in async functions. Try a `Channel` instead. |
| N0009 | Expected a context argument. Try removing the `context` attribute. |
| N0010 | Exported functions cannot receive `self`. |
| N0011 | Unsupported property. |
| N0012 | `async` attribute should not be used with an `async fn`. |
