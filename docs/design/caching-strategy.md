# Caching Strategy

**THIS DOCUMENT IS A WORK IN PROGRESS!**

## Refresh Intervals by Type

In this section we'll explore how long we'll accept the cached results before we check on whether it (or it's underlying resources) have changed.

| Type              | Interval    | Comment                             |
| ----              | --------    | -----                               |
| Document (local)  | _immediate_ | local documents with no LLM synthesis can be checked in real time |
| Document (remote) | 6 hours     | remote documents are almost always slow moving |
| LLM Synthesis     | 14 days     | after 14 days we will check the underlying sources to see if we need to run the LLM synthesis again |
| **fill in!**        | ...         | ...                                 |

## Surreal ERD

The major entities in the database and how they relate.


### Schemas


## Hashing Strategy

- **TODO** do we just hash the document in it's entirety?
    - if we do then we should at least make sure we're removing any whitespace at start and end so additional whitespace doesn't cause a "change" to occur
    - if might be work hashing each "reference block" and "everything else"
        - this would give us more visibility into WHAT changed
        - would this resolution allow us to be smarter? is it worth the extra complexity?

