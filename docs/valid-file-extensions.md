# Valid File Extensions

When we are linking to content in a Darkmatter file, the files which will be considered valid are those which are within the [file scope](./file-scope.md)
and have a file extension that we know how to deal with.

## Supported File Extensions

- **Images**
    - `.gif`
    - `.jpg` and `.jpeg`
    - `.png`
    - `.avif`
    - `.webp`
- **Documents**
    - `.md` - _a Markdown or Darkmatter document_
    - `.pdf` - _a PDF document_
    - `.txt` - _a plain text file_
- **Code Blocks**
    - `.rs` - _Rust code file_
    - `.py` - _Python code file_
    - `.ts` - _Typescript code file_
    - `.js` - _Javascript code file_
    - `.go` - _GoLang code file_
    - `.php` - _PHP code file_
    - `.pl` - _Perl code file_
    - `.sh` or `.bash` - _a Bash code file_
    - `.zsh` - _a Zsh code file_
    - `.bat` - _a Windows batch file_
    - `.json` - _a JSON data structure_
    - `.yaml` or `.yml` - _a YAML data structure_
    - `.sql` - _a SQL code block_
    - `.c` - _a C code block_
    - `.cpp` - _a C++ code block_
    - `.lua` - _a Lua code block_
- **Audio Files**
    - `.mp3`
    - `.wav`


It's important to note that not all of these file extensions will always be allowed. It depends on the [document scope](./reference/doc-scope.md) which groups of file extensions will be allowed.
