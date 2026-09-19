# gbrowse

All user data is located in the `gdata` directory. 
Saved URLs are stored in `gdata/urls`. 
User settings are defined in `gdata/init`.

#### gdata/init example:
```
timeout  = 5
init_url = "gemini://geminiprotocol.net/"
style    = "moss"
keys     = "pearbear"
```

Styles are defined in the `gdata/styles` directory. 
Keys are defined in the `gdata/keys` directory. 
In the previous example, the files `gdata/styles/moss` 
and `gdata/keys/pearbear` are selected. 


## Styles
In a styles file, you can create your own color palette 
by defining colors in a table named `palette`. This allows you 
to refer to custom colors by custom names elsewhere in the file.


#### gdata/styles/example:
```
palette = {
  black    = "#202225",
  dgrey    = "#707880",
  grey     = "#aaabb0",
  white    = "#b8a098",

  green    = "#bca47c",
  cyan     = "#709080",
  magenta  = "#c494ac",
  red      = "#e08c78",

  dmagenta = "#402c40",
  dyellow  = "#4c4844",
  dblue    = "#30343a",
}
border = {
  fg      = "dgrey", 
  bg      = "black", 
  corner  = "round", 
  bracket = "space"
}
dialog_border = {
  fg      = "grey",
  bg      = "black",
  corner  = "round",
  bracket = "space"
}
dialog_heading = {fg = "grey",   bg = "black", bold = true}
dialog_body    = {fg = "white",  bg = "black"}
banner         = {fg = "grey"}
footer         = {fg = "grey"}
general        = {fg = "white",  bg = "black"}
list           = {fg = "grey"}
preformat      = {fg = "magenta", wrap = false}
quote          = {fg = "grey"}
link           = {fg = "cyan"}
h1             = {fg = "red", underline = true}
h2             = {fg = "green", underline = true}
h3             = {fg = "green"}
text           = {wrap = true}

screen_margin        = {n = 2, s = 2, e = 16, w = 16}
text_margin          = {n = 1, s = 1, e = 4, w = 4}
```

## Keys

Key assignments don't yet support explicit modifiers like Ctrl or Alt. 
Key assignments may support explicit modifiers in the future, but it's not a 
priority. Some justification for leaving that feature out might be 
that doing so minimizes the possibility that a key assignment
collides with one used by your operating system, window manager, 
terminal emulator, terminal multiplexer, etc. For example, if you 
assigned CTL-V to the `new_tab` function, but forgot your OS or 
terminal already uses CTL-V to paste text, you wouldn't see gbrowse 
create a new tab, nor would you see any text pasted. You'd see nothing, and
that's just depressing. 
Leaving the feature out also means that when you're setting keyboard 
shortcuts for your OS or terminal or window manager (etc.), you don't have 
to worry about colliding with gbrowse. Thus, adding support for modifiers 
in key assignments is a low priority.


#### gdata/keys/example:
```
move_up     = "o"
move_down   = "i"
move_left   = "e"
move_right  = "n"

select      = "w"
delete_tab  = "v"
new_tab     = "p"

save_url    = "u"
load_url    = "U"

cycle_left  = "E"
cycle_right = "N"

yes         = "y"
no          = "n"
ack         = "y"
cancel      = "esc"
```
