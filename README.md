# wrsr-mt (v0.4)

Command-line application, providing a variety of modding-related tools for "Workers &amp; Resources: Soviet Republic".

# Features
 ### Validation
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

 ### Geometry transformations to whole mod buildings (\*.nmf and \*.ini files together). This batch transformation requires all needed files to be in the building directory - otherwise you can use the individual file manipulation operations.
   - Scaling by a given factor.
   - Mirroring.

 ### Manipulating individual mod files

   - building.ini and renderconfig.ini
     - Scaling coordinates by a given factor.
     - Mirroring coordinates.

   - \*.nmf files
     - Displaying model structure (submaterials, objects, geometry).
     - Geometry scaling (by a given factor).
     - Geometry mirroring.
     - Exporting into Wavefront's \*.obj format ([example](https://www.youtube.com/watch?v=vJ6aN4iXCas)).
 
 ### (WIP) modpacks 
   - Generating customized mods in *workshop_wip* directory, using assets from workshop mods and stock buildings.


Most subcommands support --help parameter:

    $ wrsr-mt --help
    $ wrsr-mt nmf --help
    $ wrsr-mt nmf scale --help


# Examples

Validation:

    # Check building.ini for syntax errors
    $ wrsr-mt ini parse-building  CityMagazynA/building.ini
    
    # Validate whole building in directory 'HOUSE3'
    $ wrsr-mt mod-building validate HOUSE3


Scaling/mirroring:

    # Scale the whole building in directory 'HOUSE3' (models and ini files) by x1.2
    # Store the result in directory 'HOUSE3_bigger'
    $ wrsr-mt mod-building scale HOUSE3 1.2 HOUSE3_bigger

    # Scale 'building.ini' by x1.3. Store the result in 'bigger_building.ini'
    $ wrsr-mt ini scale building building.ini 1.3 bigger_building.ini
    
    # Mirror 'model.nmf' and save it into new file 'model_mirrored.nmf'
    $ wrsr-mt nmf mirror model.nmf model_mirrored.nmf


Nmf-specific features:

    # Show details of 'model.nmf':
    $ wrsr-mt nmf show model.nmf

    # Export model geometry from 'model.nmf' into 'model.obj'
    $ wrsr-mt nmf to-obj model.nmf model.obj
    
