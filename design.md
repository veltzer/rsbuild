This is rust build tool. with incrementatl build and checksums.

The main commands are:

```bash
$ rsb build
```

This issus an incremental build.

```bash
$ rsb clean
```

This issues a full clean.

We will use the best command line parsing engine.

Config system

Config files will be in python code and in the `config` folder by conventions only.

We will have a `load_python` in tera that will load python config files from any path
and will make the config values available for templating.
The config files will usually be in a folder config beside templates.


First feature - templates

convention over configuration.
Every file in templates/{X}.tera will create a file called {X} (no templates prefix and no .tera suffix)
using the tera templating engine.

