local ok, lspconfig = pcall(require, "lspconfig")
if not ok then
	vim.notify("[dice-lsp.nvim] lspconfig not found", vim.log.levels.ERROR)
	return
end

local configs = require("lspconfig.configs")

local M = {}

function M.setup(opts)
	opts = opts or {}

	if not configs.dice_lsp then
		configs.dice_lsp = {
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
