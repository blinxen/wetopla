# Weekly todo planer

## Requirements

- vim (used for editing tasks)

## Key mappings

### Normal mode

- `Ctrl-s`: Save modifications
- `q`: Quit
- `i`: Enter input mode
    - if project container is focused then a new project can be created
    - if task container is focused then a new task can be created
- `e`: Edit task content
- `d`: Set task to done
- `<ENTER>`: Select currently highlighted project and focus task container
- `<ESC>`: Return to project container
- `<DELETE>`: Delete project or task

### Insert mode

- `<ESC>`: Cancel line input and enter normal mode
- `<ENTER>`: Create project or add task to project and enter normal mode

### Delete mode

- `<ESC>`: Cancel operation and enter normal mode
- `<Enter>`: Delete project or task and enter normal mode

### Save mode

- `<ESC>`: Cancel operation and enter normal mode
- `<Enter>`: Save changes made

### Quit mode

- `<ESC>`: Cancel operation and enter normal mode
- `<Enter>`: Quit

## License

This project is licensed unter the [MIT](./LICENSE) license.
