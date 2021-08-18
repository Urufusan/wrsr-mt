# wrsr-mt

Command-line application, providing a variety of modding-related tools for "Workers &amp; Resources: Soviet Republic".

# Features
 - Validating mods
   - Parsing and reporting syntax errors in individual configuration files (renderconfig.ini, building.ini, \*.mtl). 
     Catches most typos in token names, literals (construction phases, resources, ...), wrong amount or type of parameters.
   - Complete modded buildings. Given a path to a building directory it does the following:
       1. Parses renderconfig.ini and extracts paths to all \*.nmf and \*.mtl files;
       2. Parses \*.mtl files from step 1 and extracts paths to all textures (\*.dds);
       3. Checks that all the above references are correct (all those files exist);
       4. Parses the building.ini;
       5. Checks if any tokens in building.ini are referring to unexisting node names (using the main model's nmf as a reference).
          This includes 
           - $STORAGE_LIVING_AUTO
           - $COST_WORK_BUILDING_NODE
           - $COST_WORK_BUILDING_KEYWORD
           - $COST_WORK_VEHICLE_STATION_ACCORDING_NODE
       6. Checks if any active submaterial in the main model's nmf does not have a corresponding entry in the *.mtl files;
       7. Prints out all found issues.

 - applying geometry transformations to whole mod buildings (\*.nmf and \*.ini files together). This batch transformation requires all needed files to be in the building directory - otherwise you can use the individual file manipulation operations.
  - scaling the building by a given factor
  - (WIP) mirroring the building

 - manipulating individual mod files

  - building.ini and renderconfig.ini
   - Scaling coordinates by a given factor
   - (WIP) mirroring coordinates

  - \*.nmf files
   - displaying model structure (submaterials, objects, geometry)
   - geometry scaling (by a given factor)
   - geometry mirroring (along X-axis)
   - exporting into Wavefront's \*.obj format ([example](https://www.youtube.com/watch?v=vJ6aN4iXCas))
 
 - modpacks (generating customized mods in *workshop_wip* directory, using assets from workshop mods and stock buildings)


Most subcommands support --help parameter:

    $ wrsr-mt --help
    $ wrsr-mt nmf --help
    $ wrsr-mt nmf scale --help


# Examples

Check building ini-file for syntax errors:

    $ wrsr-mt ini parse-building  CityMagazynA/building.ini


Validate building mod in directory HOUSE3

    $ wrsr-mt mod-building validate HOUSE3


Scale the whole building in directory HOUSE3 (models and ini files) by x1.2. Store the result in directory HOUSE3\_bigger.
    
    $ wrsr-mt mod-building scale HOUSE3 1.2 HOUSE3_bigger


Scale building.ini by x1.3. Updated file will be saved as building.ini\_x1.3

    $ wrsr-mt ini scale-building building.ini 1.3

Show details of 'model.nmf':

    $ wrsr-mt nmf show model.nmf
    

Scale up *model.nmf* by 15% and save into new file *model_2.nmf*

    $ wrsr-mt nmf scale model.nmf 1.15 model_2.nmf
    

Mirror *model.nmf* along X-axis and save it into new file *model_mirrored.nmf*

    $ wrsr-mt nmf mirror-x model.nmf model_mirrored.nmf
    

Export model geometry from *model.nmf* into *model.obj*

    $ wrsr-mt nmf to-obj model.nmf model.obj
    
