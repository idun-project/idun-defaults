## idun-defaults

This repository contains scripts, programs, and configuration files that are critical to Idun's interfacing of the Commodore software (@see idun-cartridge repository) with the Linux OS running on the Raspberry Pi.

idunsh/
-------
Directory contains the Rust source code for the `idunsh` command. This Linux command-line program is used to tell the shell.app program running on the Commodore what to do. There are many sub-commands available to `idunsh` e.g. "load", "exec", "reboot", and "mount." Each sub-command invokes the shell.app to run something on the Commodore. Most of this is transparent to the user because `idunsh` is automatically run as needed by higher-level functions in the idun user's `.bashrc` file (@see bashrc below). It is worth noting that the control of the Commodore by `idunsh` works through the idun-cartridge's Lua interface, using Lua builtin functions - predominately the `sys.shell()` Lua function. This means that Lua scripts running on the Raspberry Pi can do all the things that `idunsh` can do, and do them directly.

ffetch/
-------
Directory contains the Rust source code for the `ffetch` command. This Linux command-line program scans the hardware and provides a short summary of specifications. The source has been customized for Idun. It can recognize the `idunio` process as the terminal emulator being used, and can parse the `IDUN_SYS` environment variable to detect the Commodore machine type, it's number of 64KB memory banks, and which display is used.

idunrc.toml
-----------
Here is the default configuration file needed by the idun-cartridge. The cartridge is _extensively_ configurable, and all the key parameters are found in this TOML format file. The file is human-readable, commented, and mostly designed to be hand-edited by the user for customization. The only parameters in this file that are changed by the cartridge software are the mount points for virtual drives and virtual floppies, but you may also edit those manually.

bashrc
------
This is the source for the Bash script that will be copied to `/home/idun/.bashrc` at installation. It creates a convenient command-line interface for accessing the `idunsh` functionality and controlling shell.app on the Commodore. This approach makes it very easy to extend Idun's shell interface with new commands.

newshell
--------
When the idun-cartridge first boots up, and first starts the shell.app, this Bash script is run. This is how the `ffetch` command gets invoked when the shell first appears. After that initial startup, this script is ignored by the shell.app.
