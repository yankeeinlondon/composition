# Hashing

The hashing requirements for this repo are not cryptographic and therefore we have decided to standardize on the super fast [xxHash](../.claude/skills/xx-hash/SKILL.md) hashing algorithm.

Unless stated otherwise we will always use:

- the **xxhash-rust** crate
- the **XXH3** variant with 64bits


