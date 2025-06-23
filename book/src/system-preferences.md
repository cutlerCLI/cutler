# System Preferences

cutler can do a number of things if you use it right. Here's a basic example of a TOML configuration for cutler:

```toml
[set.dock]
tilesize = 46

[set.menuextra.clock]
FlashDateSeparators = true
```

macOS heavily relies on preference files (in `.plist` format) stored in certain ways to save the state of your Mac's apps and settings. cutler takes advantage of this mechanism to automatically put your desired system settings in place by following the config file you wrote. It's a "declarative" way to set your settings without even touching the app itself.

Ideally, the block above would look something like this if you were to manually call the `defaults` CLI tool which is used to modify these values on macOS:

```bash
$ defaults write com.apple.dock "tilesize" -int "46"
$ defaults write com.apple.menuextra.clock "FlashDateSeparators" -bool true
```

You can also configure global preferences like this:

```toml
[set.NSGlobalDomain]
InitialKeyRepeat = 15
ApplePressAndHoldEnabled = true
"com.apple.mouse.linear" = true

# or, for the third entry, alternate structure:
#
# [set.NSGlobalDomain.com.apple.mouse]
# linear = true
```

Again, if you were to use `defaults`, it would look something like this:

```bash
$ defaults write NSGlobalDomain "ApplePressAndHoldEnabled" -bool true
$ defaults write NSGlobalDomain com.apple.mouse.linear -bool true
```

Once you're ready, run this command to apply everything:

```bash
$ cutler apply
```

The `apply` command has multiple functionalities which happen alongside of applying the preferences. Please read the sections below in depth to learn more.

cutler also takes the changes into account and tracks them. To see your status, run:

```bash
$ cutler status
```

Unapplying everything is also as easy. Run the command below and cutler will restore your preferences to the exact previous state:

```bash
$ cutler unapply
```

## Action Hints

The fun part about using cutler is, it will mostly tell you to take certain actions based on what command you are using, without you having to think about it. This is due to cutler's immense synchronization between commands.

Say, for example, if my Dock should be at the `right` based on cutler's configuration, and `bottom` is its actual value right now, cutler will show this when running `cutler status`:

```bash
$ cutler status
[WARNING] com.apple.dock | orientation: should be right (currently bottom)
[WARNING] Run `cutler apply` to apply these changes from your config.

🍎 All Homebrew formulae/casks match config.
```

As you can see, it suggests me to run `cutler apply`. Running the suggested command will only affect the changed portion of the preferences, and cutler will skip the rest.

## Risky Operations

If you would like to write non-existent domains (create them) using cutler, you can run the `apply` command with a particular flag:

```bash
$ cutler apply --disable-checks
```

This will disable the "Domain does not exist" error which happens when cutler's backend does not recognize a domain.
