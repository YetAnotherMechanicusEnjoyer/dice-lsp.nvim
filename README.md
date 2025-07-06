# Dice Language Server Protocol

<img src="https://i.ibb.co/rRbK7fd7/Screenshot-20250706-061600.png" alt="Example Image" border="0">

## Table of Content

- [About](#about)
- [Installation](#installation)
  * [Dependencies](#dependencies)
  * [LSP](#lsp)
- [Neovim](#neovim)
  * [Lazy](#lazynvim)
  * [Packer](#packernvim)
  * [Vim Plug](#vim-plug)
- [License](#license)

## About

> [!NOTE]
> A language server protocol Neovim plugin for `.dice` files

## Installation

### Dependencies

> [!IMPORTANT]
> Make sure to have [Rust](https://www.rust-lang.org/tools/install) installed.

### LSP

> [!TIP]
> Clone the repo somewhere, compile the LSP with [Cargo](https://doc.rust-lang.org/cargo/) and make a simlink to `/usr/bin/`.

```bash
cargo build --release
cargo run --release
sudo ln -s /full/path/to/target/release/dice-lsp /usr/bin/dice-lsp
```

## Neovim

Copy and Paste the code block corresponding to your neovim config's plugin manager.

### lazy.nvim
```lua
{
  "YetAnotherMechanicusEnjoyer/dice-lsp.nvim",
  dependencies = {
    "neovim/nvim-lspconfig",
  },
  config = function()
    require("dice_lsp").setup()
  end,
}
```

### packer.nvim
```lua
use {
  "YetAnotherMechanicusEnjoyer/dice-lsp.nvim",
  requires = {
    "neovim/nvim-lspconfig"
  },
  config = function()
    require("dice_lsp").setup()
  end
}
```

### vim-plug
`init.vim` or `init.lua`
```vim
Plug 'neovim/nvim-lspconfig'
Plug 'YetAnotherMechanicusEnjoyer/dice-lsp.nvim'
```
`init.lua`
```lua
require("dice-lsp").setup()
```

## License
[MIT](https://github.com/YetAnotherMechanicusEnjoyer/dice-lsp.nvim/blob/1c3ceb81c90186b23b308ba43b2856474bfccea2/LICENSE)
