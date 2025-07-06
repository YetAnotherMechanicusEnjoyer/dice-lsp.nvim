local lspconfig = require("lspconfig")

local M = {}

function M.setup(opts)
	opts = opts or {}

	lspconfig.dice_lsp = {
		default_config = {
			cmd = opts.cmd or { "dicec" },
			filetype = { "dice" },
			root_dir = lspconfig.util.root_pattern(".git", "."),
			settings = opts.settings or {},
		},
	}

	lspconfig.dice_lsp.setup(opts)
end

return M
