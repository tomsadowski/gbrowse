## gbrowse

All user data is located in the `gdata` directory. Saved URLs are stored in `gdata/urls`. User settings are defined in `gdata/init`.

#### gdata/init example:
```
init_url = "gemini://geminiprotocol.net/"
style    = "fire"
keys     = "arrrows"
```

Styles are defined in the `gdata/styles` directory. Keys are defined in the `gdata/keys` directory. In the previous example, the files `gdata/styles/fire` and `gdata/keys/arrows` are selected.

#### styles example:
```
screen_margin = {n = 1, s = 0, e = 8, w = 8}
text_margin   = {n = 1, s = 1, e = 4, w = 4}

border    = {fg = "585c60", bg = "202326", corner = "round", bracket = "tort"}
covered   = {fg = "303438"}
banner    = {fg = "e088f0"}
general   = {fg = "e8e8e8", bg = "202326"}
list      = {fg = "e8e8e8"}
info      = {fg = "e8e8e8"}
quote     = {fg = "e088f0"}
preformat = {fg = "e088f0", wrap = false}
link      = {fg = "80d0e0", underline = true}
header1   = {fg = "e0b040", bold = true, underline = true}
header2   = {fg = "ff6060", bold = true}
header3   = {fg = "ff6060"}
```

#### keys example:
```
move_up     = "up"
move_down   = "down"
move_left   = "left"
move_right  = "right"

select      = "enter"
delete_tab  = "r"
new_tab     = "t"

save_url    = "u"
load_url    = "U"

cycle_left  = ","
cycle_right = "."

yes         = "y"
no          = "n"
ack         = "enter"
cancel      = "esc"
```
