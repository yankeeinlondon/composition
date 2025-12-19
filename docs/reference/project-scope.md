# Project Scope

The _project scope_ represents the files which are considered relevant to the library module when performing operations.

- from the perspective of the **Library Module** this is derived from the directory which is passed into the initial call to the `init(dir)` function.
- from the perspective of the **CLI Module** and the **LSP Module** this is derived from the _current working directory_

In all cases this reference directory will be evaluated for whether _it is_ or _is not_ part of a **git** repository.

- if it **is** part of a git repository then
    - all files -- except those files eliminated by the `.gitignore` file -- are consider the project scope.
    - the database which will be used will be repo specific and stored in `{{repo root}}/.composition.db`
- if it **is not** part of a git repository then
    - the files in and underneath the reference directory are considered the project scope.
    - the database which will be used will be user specific and stored in `${HOME}/.composition.db`

> **Note:** while the **Project Scope** determines the set of relevant files we can reference locally, the actual files which are valid are based on context and this will largely be determined by [document scope](./doc-scope.md).


