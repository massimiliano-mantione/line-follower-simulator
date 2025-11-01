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
config.default_cwd = '/home/massi/data/massi/bots/line-follower-simulator/presentation'

--config.color_scheme = 'Batman'

return config
