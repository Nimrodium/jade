# Overview
Jade is a declarative agnostic mod manager centered for minecraft, but can be used for any scenario.
it is built for constantly evolving modpacks, where instead of the mods/ folder being the modpack, the jade manifestfile is the modpack, unlike other mod managers, where a pack can be snapshotted, exported, and then imported, the jademanifest is instead always the source of truth. and the mods/ folder is just the evaluation of that file. much like Nix.
this allows for easy modpack syncing with other players with packs that are constntly evolving, encourages additions without having to re-export, re-upload, and rebuild your entire instance with the new pack, or manually adding the mod, in either case you must still inform everyone what mods changed and how to update to play on the server, with Jade, this process can be automated.


jade manager v2 differs from jade manager v1 in a few key implementation details. mostly internal code refactoring, but externally,
it moves away from a directory based approach and instead uses a single manifest `<name>.jademanifest` or `<name>.jade`, as well as a `<name>.jadelock`

# Jademanifest
a jade manifest is the source of truth for Jade, it contains at least 2 files. the manifest, which can be named `<name>.jademanifest`, `<name>.jade`,

# Deriving a package
To Derive a package (mod) using jade, two things are needed

a `PackageFormula` and a `PackageDescription`
## Package Formula
a package formula describes how to unpack and deploy a package
it describes the downloaded file's format, if it's a zip, it describes the path to the target in the zip.
it describes the deployment target path as well.

```toml
    [[Formula]]
    name="ModrinthPkg"
    input="file"
    target="mods/"
    [[Formula]]
    name="CkanPkg"
    input="zip"
    zip_target="GameData/"
    zip_target_dirwild=true # makes GameData/ move all subfolders / files instead of the whole directory
    target="GameData/"
```


## Package Description
this describes an instance of a package, it sources the package formula for its default deployment options,
but all options in a formula can be overridden here.

  ```toml
  [[Package]]
  driver="ModrinthPkg"
  pkg="sodium"
  ver="latest"
  # or direct url
  # url=https://cdn.modrinth.com/data/AANobbMI/versions/1LjoeVdt/sodium-neoforge-0.7.0%2Bmc1.21.8.jar
  ```


## Package Source Driver
Any package (mod) source needs to know how to fetch the package from the webserver (or any other source), the default drivers are
	* Modrinth
	> obtain a mod.jar file from modrinth
	* File
	> obtain a generic file from the local file system
	* Jademanifest
	> obtain an artifact from the jade manifest itself, similar to file but ensures that the local directory is the same as the manifest.
	* Github
	> obtain a file from github.

# conditional tag evaluation
tags can be used for conditional evaluation of a manifest.

## examples
* server
> server specific mods
* client
> client specific mods
* optional
> optional (entirely client side mods)
* shader
> shaderpacks
* config
> configuration files
* lib
> a library mod.

during compose-time the --derive flag can be used to conditionally build a config, using a comma seperated list:
`jade compose name.jade --derive client,shader,config,optional`
an exclude variant, `--exclude` can be also included
`jade compose name.jade --exlude server,config,optional`
a default list of tags for when derive and or exclude are not included can be set in the manifestfile.
```toml
    default_tags=["client"]
```
a meta-tag called `all` is included which acts as a wildcard.

conditional tag evaluation uses `depends=[...]` to ensure that an included package drags in its dependencies,
if a dependency is excluded but is required by a package that is included, the dependency will be included, however a warning will be raised.

# cli interface:
## interact with full manifests
	* `jade compose <name.jade>`
	> evaluate a jade manifest
	* `jade export <name.jade> <name.jadelock>`
	> export a `<name>.jade.zip` file
	* `jade import <name.jade.zip>`
	> imports a `jade.zip` and deploy.
	* `jade delete <installed manifest>`
	> marks manifest as dead in garbage collection
	* `jade info <name.jade>`
	> shows info about a package.
## modifying a manifest
	* `jade <name.jade> add <package>`
	* `jade <name.jade> search <package>`
	* `jade <name.jade> rm <package>`
	* `jade <name.jade> info <package>`
	* `jade <name.jadelock> update`
	* `jade <name.jade> add-tag <pkg> <tag>`
	* `jade <name.jade> rm-tag <pkg> <tag>`


## misc
	* `jade version`
	> get version info and compiled source drivers
	## flags
	* `--target`
	> evaluation target
	*`--fetch` (or force-fetch?)
	> forces jade to download from the package source and replace store path even it is already present
	* `--verbose`
	> verbose print

# launching minecraft
Jade may or may not include an inbuilt launcher to launch minecraft or generally a thing.
likely have a Launcher trait and impls, and each implementation are compiled in and available as modules,

# GTK UI
Jade may or may not include an inbuilt GUI using GTK, likely will be built using Glade.

# Specific Integration
```toml
mc_ver="1.21.1"
mc_mod_loader="forge"
```

# Disk
Jade stores runtime and persistant data at:
  * `%APPDATA%\jade\` (Windows),
  * `~/Library/Application Support/` (MacOS)
  * `~/.local/share/jade/` (Linux)

internally jade is organized as such:
  * `jade/store`
  > registry of all packages
  * `jade/var`
  > variable data
  * `jade/var/manifests/`
  > index of all manifests
  * `jade/var/manifests/*/generations/0../`
  > stack of manifest generations, for rollback.
  > index of all dead manifests (deployed but marked dead)
  * `jade/var/manifests/*/generations/*/<name>.manifest.jade`
  > copied manifest
  * `jade/var/manifests/*/generations/*/<name>.lockfile.jade.lock`
  > copied lock file
  * `jade/var/manifests/*/generations/*/derived.txt`
  > list of store paths used by this manifest

  * `jade/var/eval`
  > evaluated deployments of a manifest, you do not have to use eval, it is simply a place made available.

jade writes packages which havent been copied to the store to `$TMP/jade/pkgs/` (MacOS/Linux) and `%TMP%\jade\pkgs\` (Windows).
-# for MacOS/Linux it will fall back to /tmp if $TMP is not set.


## Store
the store works very similarly to `/nix/store`, it is a content addressed immutable and atomic store of derived packages.
all derivations in the store are by default symlinked to their target. to allow for deduplication.

When a manifest is evaluated first it checks `jade/var` and checks the generation hash to the live manifest hash. if they dont match, then it evalutes the manifest, if they match then it checks if the lock file has changed. if it has it will evaluate anyways
for every package in the manifest it checks if the values in the lockfile are the same and if so then it switches the lockfile which provides a direct cached download link. and hash. if the manifest and lockfile are different then it derives the download link from the manifest, hashes the artifact, then writes an entry in the lockfile. after it finishes it moves the artifact from TMP to its store path
