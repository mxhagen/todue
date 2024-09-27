
# ☑️  todue

Keep track of TODOs and deadlines using an interactive markdown TUI.

---

### Layout preview

This is roughly what the UI looks like in the terminal

```md
  [todue]                Chores
  [x] (2024-06-20 16:00) Get groceries
  [ ] (2024-06-20 20:00) Do the dishes
  [ ] (2024-06-20 21:00) Take out the trash
```

### Control scheme

The control scheme is vim-like and features a minimal line editor as well as datetime-input.

- `j`/`k`: move focus down/up
- `J`/`K`: move focused entry down/up
- `<space>`: toggle focused entry completed
- `q`: save and quit
- `Q`: quit without saving
- `g`/`G`: move focus to top/bottom
- `s`: cycle sort mode. 


### TODO

Things that might be implemented in the future

- more controls
    - `a`/`A`: append to entry text (enters line editor)
    - `c`/`C`: change entry text (enters line editor)
    - `i`/`I`: insert before entry text (enters line editor)
    - `o`/`O`: edit new entry (after/before current - enters line then datetime editor)
    - `r`: replace entry
    - `/`/`?`: search entry by text (backwards) (wrapping)
        - later on regex search
    - `u`/`<ctrl-z>`: undo
    - `<ctrl-r>`/`<ctrl-y>`: redo
    - `z`: collapse/expand current group
    - `yd`: copy entry date
    - `yt`: copy entry text
    - `yy`: copy entire entry
    - `0`-`9`: as prefix for repeated commands

- line editor with vim commands (prefixed with mode)
    - normal: `<esc>`: exit line editor
    - normal: `a`/`A`: append (to end)
    - normal: `i`/`I`: insert (at beginning)
    - normal: `d`: delete
    - normal: `x`: remove character
    - normal: `c`/`C`: change
    - normal: `r`/`R`: replace
    - normal: `s`/`S`: substitute (equal to `cl` and `cc` respectively)
    - normal: `v`: visual mode
    - normal: `y`/`Y`: copy
    - normal: `p`/`P`: paste
    - normal: `u`/`<ctrl-z>`: undo
    - normal: `<ctrl-r>`/`<ctrl-y>`: redo
    - normal: `f`/`F` and `t`/`T`: find (until) (backwards)
    - normal: `/`/`?`: search (backwards) (wrapping to beginning of line)
        - later on regex search
    - insert: `w`,`b`,`e`: like vim, including uppercase equivalent
    - insert: `<esc>`: exit line editor
    - insert: `<ctrl-w>`/`<ctrl-backspace>`: delete last word
    - insert: `<ctrl-shift-v>`/`<shift-insert>`: paste
    - visual: `a`/`i`: select all/inside of...

- datetime editor
    - highlight date part (YYYY for example)
    - `d`: remove entire deadline
    - `<enter>`: go to next part
    - `0`-`9`: input number (ignoring invalid inputs like months >12)

- sort mode: cycle through modes and set ascending/descending separately
    - `r`: insert before entry text (enters line editor)

- collapsable todo group hierarchy
    - detect indent width from md
    - group entries together under previous entry with lower indent level
    - display expandable groups in tui

- config
    - keybinds
    - some other options (?)

- some `:`-commands?
    - regex substitution
    - set commands for config entries
    - help command that shows controls
    - `set` with no key shows explicitly set keys (config and live)
    - config wizard & write current config state to config file
