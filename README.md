# wrsr-mt

Command-line application, providing a variety of modding-related tools for "Workers &amp; Resources: Soviet Republic".

# Features
 - manipulating *\*.nmf* files
   - displaying model structure (submaterials, objects, geometry)
   - scaling
   - mirroring
   - exporting into Wavefront's \*.obj format
   - deleting objects
 
 - modpacks (generating customized mods in *workshop_wip* directory, using assets from workshop mods and stock buildings)


# Examples

Show details of 'model.nmf':

    $ wrsr-mt nmf show model.nmf
    

Scale up *model.nmf* by 15% and save into new file *model_2.nmf*

    $ wrsr-mt nmf scale model.nmf 1.15 model_2.nmf
    

Mirror *model.nmf* along X-axis and save it into new file *model_mirrored.nmf*

    $ wrsr-mt nmf mirror-x model.nmf model_mirrored.nmf
    

Export model geometry from *model.nmf* into *model.obj* ([example](https://www.youtube.com/watch?v=vJ6aN4iXCas))

    $ wrsr-mt nmf to-obj model.nmf model.obj
    

Most subcommands can show usage help when used with parameter --help 

    $ wrsr-mt --help
    $ wrsr-mt nmf --help
    $ wrsr-mt nmf scale --help
