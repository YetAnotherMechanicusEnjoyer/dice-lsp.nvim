local lspconfig = require("lspconfig")

local M = {}

function M.setup(opts)
	opts = opts or {}

	if not lspconfig.configs.dice_lsp then
		lspconfig.configs.dice_lsp = {
			default_config = {
				cmd = opts.cmd or { "dice-lsp" },
				filetypes = { "dice" },
				root_dir = lspconfig.util.root_pattern(".git", "."),
				settings = opts.settings or {},
			},
		}
	end

	lspconfig.dice_lsp.setup(opts)
end

return M
