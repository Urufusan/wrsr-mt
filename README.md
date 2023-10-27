## wrsr-mt (v0.5.1)

Command-line application, providing a variety of modding-related tools for "Workers &amp; Resources: Soviet Republic".

## Features

#### Validation

- Parsing and reporting syntax errors in individual configuration files (renderconfig.ini, building.ini, \*.mtl).
  Catches most typos in token names, literals (construction phases, resources, ...), wrong amount or type of parameters.
- Complete modded buildings. Given a path to a building directory it does the following:
  1. Parses renderconfig.ini and extracts paths to all \*.nmf and \*.mtl files.
  2. Parses \*.mtl files from step 1 and extracts paths to all textures (\*.dds).
  3. Checks that all the above references are correct (all those files exist).
  4. Parses the building.ini.
  5. Checks if any tokens in building.ini are referring to unexisting node names (using the main model's nmf as a reference).
     This includes
     - $STORAGE_LIVING_AUTO
     - $COST_WORK_BUILDING_NODE
     - $COST_WORK_BUILDING_KEYWORD
     - $COST_WORK_VEHICLE_STATION_ACCORDING_NODE
  6. Checks if any active submaterial in the main model's nmf does not have a corresponding entry in the *.mtl files.
  7. Prints out all found issues.

#### Geometry transformations (whole building in one operation)

Applicable to whole mod buildings (\*.nmf and \*.ini files together). These transformations requires all needed files to be in the building directory - otherwise you can use the individual file manipulation operations.

- Scaling by a given factor.
- Mirroring.

#### Manipulating individual mod files

- building.ini and renderconfig.ini
  
  - Scaling coordinates by a given factor.
  - Mirroring coordinates.
- \*.nmf files
  
  - Displaying model structure (submaterials, objects, geometry).
  - Geometry scaling (by a given factor).
  - Geometry mirroring.
  - Optimizing faces' indices (reducing vertex data duplication)
  - Exporting into Wavefront's \*.obj format ([example](https://www.youtube.com/watch?v=vJ6aN4iXCas)).

#### Modpacks

- Generating customized mods in*workshop_wip* directory, using assets from workshop mods and stock buildings.

Most subcommands support --help parameter:

```bash
$ wrsr-mt --help
$ wrsr-mt nmf --help
$ wrsr-mt nmf mirror --help
```

## Known issues

The following tokens in building.ini are not implemented (they will be reported as 'unknown token'). They might be added if I get a good working example of their usage.

- $STORAGE_DEMAND_MEDIUM
- $ROADVEHICLE_FORKLIFT_PASS
- $CONNECTIONS_PEDESTRIAN_DEAD_SQUARE
- $CONNECTION_AIRPLANE
- $CONNECTION_RAIL_DEAD
- $CONNECTION_ROAD_HEIGHT
- $MONUMENT_ELETRIC_CONSUMPTION_ADD
- $WORKING_SFX_DISTANCE
- $ANIMATION_MESH_WORKSHOP

## Examples

Validation:

```bash
# Check building.ini for syntax errors
$ wrsr-mt ini parse building CityMagazynA/building.ini

# Validate whole building in directory 'HOUSE3'
$ wrsr-mt mod-building validate HOUSE3
```

Scaling/mirroring:

```bash
# Scale the whole building in directory 'HOUSE3' (models and ini files) by x1.2
# Store the result in directory 'HOUSE3_bigger'
$ wrsr-mt mod-building scale HOUSE3 1.2 HOUSE3_bigger

# Scale 'building.ini' by x1.3. Store the result in 'bigger_building.ini'
$ wrsr-mt ini scale building building.ini 1.3 bigger_building.ini

# Mirror 'model.nmf' and save it into new file 'model_mirrored.nmf'
$ wrsr-mt nmf mirror model.nmf model_mirrored.nmf
```

Nmf-specific features:

```bash
# Show details of 'model.nmf':
$ wrsr-mt nmf show model.nmf

# Export model geometry from 'model.nmf' into 'model.obj'
$ wrsr-mt nmf to-obj model.nmf model.obj
```

## Important note

This project was reuploaded by me (Urufusan) because the original repo doesn't exist anymore. I'm guessing that this is the new "official" repo for this project.

