local wezterm = require 'wezterm'
local config = {}

config.keys = {
  {
    key = 'F11',
    mods = '',
    action = wezterm.action.ToggleFullScreen,
  },
}

config.font = wezterm.font 'FiraCode Nerd Font Mono'
config.font_size = 22
config.hide_tab_bar_if_only_one_tab = true
-- The directory holding this file, i.e. the slides (term.sh also passes --cwd).
config.default_cwd = wezterm.config_dir

--config.color_scheme = 'Batman'

return config
