# Editor Integration

## VS Code

VS Code has built-in LSP support through its extension API.

### Extension Structure
```
my-extension/
  client/           # Language client (VS Code extension)
    src/extension.ts
  server/           # Language server
    src/server.ts
  package.json
```

### package.json Configuration
```json
{
  "name": "my-language-extension",
  "activationEvents": ["onLanguage:mylang"],
  "contributes": {
    "languages": [{
      "id": "mylang",
      "extensions": [".mylang"],
      "configuration": "./language-configuration.json"
    }]
  },
  "main": "./client/out/extension.js"
}
```

### VS Code Settings for LSP
```json
{
  "mylang.trace.server": "verbose",
  "mylang.maxNumberOfProblems": 100
}
```

## Neovim

Native LSP support via `nvim-lspconfig`.

### Basic Configuration
```lua
-- Enable a language server
vim.lsp.enable('mylsp')

-- Configure the server
vim.lsp.config('mylsp', {
  cmd = {'my-language-server', '--stdio'},
  filetypes = {'mylang'},
  root_dir = vim.fs.root(0, {'.git', 'config.json'}),
  settings = {
    mylsp = {
      formatting = { enabled = true },
      diagnostics = { enabled = true },
    },
  },
})
```

### Custom Server Definition
```lua
-- If server isn't in nvim-lspconfig
vim.lsp.config('custom_server', {
  default_config = {
    cmd = {'/path/to/server'},
    filetypes = {'customlang'},
    root_dir = function(fname)
      return vim.fs.dirname(fname)
    end,
  },
})
```

### Keybindings
```lua
vim.api.nvim_create_autocmd('LspAttach', {
  callback = function(args)
    local opts = { buffer = args.buf }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '[d', vim.diagnostic.goto_prev, opts)
    vim.keymap.set('n', ']d', vim.diagnostic.goto_next, opts)
  end,
})
```

### null-ls / none-ls (for non-LSP tools)
```lua
-- Integrate linters/formatters that don't speak LSP
local null_ls = require('null-ls')

null_ls.setup({
  sources = {
    null_ls.builtins.formatting.prettier,
    null_ls.builtins.diagnostics.eslint,
  },
})
```

## Helix

Built-in LSP support, configured via `languages.toml`.

### Configuration
```toml
# ~/.config/helix/languages.toml

# Define language server
[language-server.mylsp]
command = "my-language-server"
args = ["--stdio"]

# Assign to language
[[language]]
name = "mylang"
scope = "source.mylang"
file-types = ["mylang"]
language-servers = ["mylsp"]
roots = [".git", "config.json"]
```

### Multiple Servers
```toml
[[language]]
name = "typescript"
language-servers = ["vtsls", "eslint"]
```

### Server-Specific Settings
```toml
[language-server.vtsls.config]
vtsls.enableMoveToFileCodeAction = true
typescript.inlayHints.parameterNames.enabled = "all"
```

## Vim (via vim-lsp or coc.nvim)

### vim-lsp
```vim
" Register server
if executable('my-language-server')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'mylsp',
    \ 'cmd': {server_info->['my-language-server', '--stdio']},
    \ 'allowlist': ['mylang'],
    \ })
endif

" Keybindings
function! s:on_lsp_buffer_enabled() abort
  setlocal omnifunc=lsp#complete
  nmap <buffer> gd <plug>(lsp-definition)
  nmap <buffer> K <plug>(lsp-hover)
endfunction
```

### coc.nvim
```json
// coc-settings.json
{
  "languageserver": {
    "mylsp": {
      "command": "my-language-server",
      "args": ["--stdio"],
      "filetypes": ["mylang"],
      "rootPatterns": [".git"]
    }
  }
}
```

## Emacs (lsp-mode or eglot)

### lsp-mode
```elisp
(use-package lsp-mode
  :hook ((mylang-mode . lsp-deferred))
  :commands lsp)

(lsp-register-client
  (make-lsp-client
    :new-connection (lsp-stdio-connection '("my-language-server" "--stdio"))
    :major-modes '(mylang-mode)
    :server-id 'mylsp))
```

### eglot (built-in)
```elisp
(add-to-list 'eglot-server-programs
             '(mylang-mode . ("my-language-server" "--stdio")))
```

## Implementation Differences

| Feature | VS Code | Neovim | Helix |
|---------|---------|--------|-------|
| LSP Client | Extension-based | Built-in | Built-in |
| Configuration | JSON + UI | Lua | TOML |
| Server Management | Automatic | Manual | Manual |
| External Tools | Extensions | null-ls | Limited |
| Diagnostics Display | Inline + Panel | Virtual text | Inline |

## Debugging Tips

### Enable Trace Logging
```lua
-- Neovim
vim.lsp.set_log_level("debug")
-- Log at: ~/.local/state/nvim/lsp.log
```

```json
// VS Code settings.json
{
  "mylang.trace.server": "verbose"
}
```

### Test with Multiple Editors
Different editors may interpret LSP responses differently. Test your server with:
1. VS Code (reference implementation)
2. Neovim (different LSP client)
3. Helix (minimal, strict)
