//use std::env;
use std::fmt::{self, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::io::Read;
use std::collections::HashMap;
use std::ops::Range;

use regex::Regex;
use lazy_static::lazy_static;



#[derive(Debug, Clone)]
enum RenderConfig<'ini> {
    Stock(&'ini str, &'ini str),
    Mod(PathBuf)
}

impl fmt::Display for RenderConfig<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderConfig::Stock(key, _) => write!(f, "Stock '{}'", key),
            RenderConfig::Mod(path) => write!(f, "Mod '{}'", path.to_str().unwrap())
        }
    }
}

#[derive(Debug, Clone)]
struct BuildingDef<'ini> {
    render_config: RenderConfig<'ini>,

    building_ini: PathBuf,
    bbox: PathBuf,
    fire: Option<PathBuf>,

    model: PathBuf,
    model_lod1: Option<PathBuf>,
    model_lod2: Option<PathBuf>,
    model_emissive: Option<PathBuf>,

    material: PathBuf,
    material_emissive: Option<PathBuf>,
}

impl BuildingDef<'_> {
    fn validate_paths(&self) {
        assert!(self.building_ini.exists());
        assert!(self.bbox.exists());
        assert!(path_option_valid(&self.fire));

        assert!(self.model.exists());
        assert!(path_option_valid(&self.model_lod1));
        assert!(path_option_valid(&self.model_lod2));
        assert!(path_option_valid(&self.model_emissive));

        assert!(self.material.exists());
        assert!(path_option_valid(&self.material_emissive));

        // TODO: parse and validate texture paths
    }
}

#[inline]
fn print_path_option(o: &Option<PathBuf>) -> String {
    match o {
        None => String::from("<none>"),
        Some(ref p) => String::from(p.to_str().unwrap())
    }
}

impl fmt::Display for BuildingDef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "\ //"BuildingDef {{\n\
        write!(f, "\
        {indent}renderconfig      {}\n\
        {indent}building_ini      {}\n\
        {indent}bbox              {}\n\
        {indent}fire              {}\n\
        {indent}model             {}\n\
        {indent}model_lod1        {}\n\
        {indent}model_lod2        {}\n\
        {indent}model_emissive    {}\n\
        {indent}material          {}\n\
        {indent}material_emissive {}\n\
        ", //}}",
        self.render_config, 
        self.building_ini.to_str().unwrap(),
        self.bbox.to_str().unwrap(),
        print_path_option(&self.fire),
        self.model.to_str().unwrap(),
        print_path_option(&self.model_lod1),
        print_path_option(&self.model_lod2),
        print_path_option(&self.model_emissive),
        self.material.to_str().unwrap(),
        print_path_option(&self.material_emissive),
        indent = "   " 
        )
    }
}


#[derive(Debug)]
enum StockBuilding<'a> {
    Unparsed(&'a str),
    Parsed(BuildingDef<'a>)
}

type StockBuildingsMap<'ini> = HashMap<&'ini str, (&'ini str, StockBuilding<'ini>)>;

#[derive(Debug)]
enum SourceType<'a> {
    Stock(&'a str),
    Mod(PathBuf),
}


#[derive(Debug)]
struct Style<'ini> {
    prefix: String,
    name: String,
    buildings: Vec<BuildingDef<'ini>>
}

impl<'ini> Style<'ini> {
    fn new<'a, 'b>(pfx: &'a str, nm: &'b str) -> Style<'ini> {
        Style {
            prefix: String::from(pfx),
            name: String::from(nm),
            buildings: Vec::with_capacity(0)
        }
    }
}


#[derive(Debug)]
struct Category<'ini> {
    prefix: String,
    name: String,
    styles: Vec<Style<'ini>>
}

impl<'ini> Category<'ini> {
    fn new<'a, 'b>(pfx: &'a str, nm: &'b str) -> Category<'ini> {
        Category {
            prefix: String::from(pfx),
            name: String::from(nm),
            styles: Vec::with_capacity(0)
        }
    }
}

// TODO ensure these are absolute
static ROOT_STOCK: &str = r"Z:\media_soviet";
static ROOT_MODS: &str = r"C:\Program Files (x86)\Steam\steamapps\workshop\content\784150"; 

lazy_static! {
    static ref PATH_ROOT_STOCK: PathBuf = PathBuf::from(ROOT_STOCK);
    static ref PATH_ROOT_MODS: PathBuf = PathBuf::from(ROOT_MODS);
}


fn main() {
    //let args: Vec<String> = env::args().collect();
    //let src = args.get(1).unwrap();
    //let dest = args.get(2).unwrap();
    
    //let tmpbuf: PathBuf = ["c:/mods", r"1234567890/boo"].iter().collect();
    //println!("{:?}", tmpbuf);


    // TODO ensure these are absolute
    let src = r"c:\projects\rbp_pack";
    let dest = r"c:\projects\rbp_generated";

    println!("Pack source: {}", src);
    println!("Installing to: {}", dest);

    let mut pathbuf: PathBuf = [ROOT_STOCK, "buildings", "buildingtypes.ini"].iter().collect();

    let stock_buildings_ini = fs::read_to_string(&pathbuf).unwrap();
    let mut stock_buildings = { 
        let mut mp = HashMap::with_capacity(512);
        let rx = Regex::new(r"\$TYPE ([_[:alnum:]]+?)\r\n((?s).+?\n END\r\n)").unwrap();

        for caps in rx.captures_iter(&stock_buildings_ini) {
            let key = caps.get(1).unwrap().as_str();
            mp.insert(
                key, 
                (key, StockBuilding::Unparsed(caps.get(2).unwrap().as_str()))
            );
        }
        
        mp
    };

    println!("Found {} stock buildings", stock_buildings.len());

    pathbuf.push(src);
    println!("Reading sources...");
    let data = read_validate_sources(pathbuf.as_path(), &mut stock_buildings);
    println!("Sources verified.");


    println!("Creating mods...");
    pathbuf.push(dest);

    generate_mods(pathbuf.as_path(), data);

}


fn generate_mods<'ini>(dest: &Path, data: Vec<Category<'ini>>) {
    let mut pathbuf = dest.to_path_buf();
    let mut pathbuf_models = pathbuf.clone();
    pathbuf_models.push("nmf");

    fs::create_dir_all(&pathbuf).unwrap();
    fs::create_dir_all(&pathbuf_models).unwrap();

    const WSH_CFG_1: &str = "$ITEM_ID ";
    const WSH_CFG_2: &str = "\n\
    $OWNER_ID 12345678901234567\n\
    $ITEM_TYPE WORKSHOP_ITEMTYPE_BUILDING\n\
    $VISIBILITY 2\n\n";
    const WSH_CFG_3: &str = "\n\
    $ITEM_NAME \"generated\"\n\
    $ITEM_DECS \"generated descr\"\n\
    $END";

    let mut buf_workshopconfig = String::with_capacity(1024);
    buf_workshopconfig.push_str(WSH_CFG_1);

    let mut buf_str_mod = String::with_capacity(64);
    let mut buf_str_bld = String::with_capacity(2);

    let mut buf_assets = Vec::<u8>::with_capacity(1024*1024*64);

    for (i_cat, category) in data.iter().enumerate() {
        write!(&mut buf_str_mod, "{:0>2}", i_cat).unwrap();

        for (i_stl, style) in category.styles.iter().enumerate() {
            write!(&mut buf_str_mod, "{:0>2}", i_stl).unwrap();

            // No more than MOD_BUILDINGS_MAX buildings in a single mod
            const MOD_BUILDINGS_MAX: u8 = 16;
            let mut i_mod = 0u16;
            let mut cnt = 0u8;

            assert!(style.buildings.len() > 0);
            for building in style.buildings.iter() {
                if cnt == 0 {
                    // setup new mod folder for this building
                    buf_str_mod.truncate(4);
                    write!(&mut buf_str_mod, "{:0>4}", i_mod).unwrap();
                    pathbuf.push(&buf_str_mod);
                    fs::create_dir(&pathbuf).unwrap();

                    // TODO: mod root files
                    buf_workshopconfig.truncate(WSH_CFG_1.len());
                    buf_workshopconfig.push_str(&buf_str_mod);
                    buf_workshopconfig.push_str(WSH_CFG_2);
                }


                buf_str_bld.clear();
                write!(&mut buf_str_bld, "{:0>2}", cnt).unwrap();
                pathbuf.push(&buf_str_bld); // mod dir -> building subdir
                fs::create_dir(&pathbuf).unwrap();

                write!(&mut buf_workshopconfig, "$OBJECT_BUILDING {}\n", &buf_str_bld).unwrap();

                //------------------------------
                // TODO: install building files

                copy_file(&building.building_ini, &mut pathbuf, "building.ini");
                copy_file(&building.bbox, &mut pathbuf, "building.bbox");
                copy_file_opt(&building.fire, &mut pathbuf, "building.fire");

                let new_model = copy_asset_md5(&building.model, &pathbuf_models, &mut buf_assets, "nmf");
                let new_model_lod1 = copy_asset_md5_opt(&building.model_lod1, &pathbuf_models, &mut buf_assets, "nmf");
                let new_model_lod2 = copy_asset_md5_opt(&building.model_lod2, &pathbuf_models, &mut buf_assets, "nmf");
                let new_model_emissive = copy_asset_md5_opt(&building.model_emissive, &pathbuf_models, &mut buf_assets, "nmf");

                //------------------------------

                pathbuf.pop(); // building subdir -> mod dir

                cnt += 1;
                if cnt == MOD_BUILDINGS_MAX {
                    cnt = 0;
                    i_mod += 1;

                    buf_workshopconfig.push_str(WSH_CFG_3);
                    pathbuf.push("workshopconfig.ini");
                    fs::write(&pathbuf, &buf_workshopconfig).unwrap();
                    pathbuf.pop();

                    pathbuf.pop(); // mod dir -> root
                }
            }

            if cnt != 0 {
                buf_workshopconfig.push_str(WSH_CFG_3);
                pathbuf.push("workshopconfig.ini");
                fs::write(&pathbuf, &buf_workshopconfig).unwrap();
                pathbuf.pop();

                pathbuf.pop(); // mod dir -> root
            }


            buf_str_mod.truncate(2);
        }

        buf_str_mod.clear();
    }

}


fn read_validate_sources<'ini>(src: &Path, stock_buildings: &mut StockBuildingsMap<'ini>) -> Vec<Category<'ini>> {

    let mut buf_sources = String::with_capacity(512);
    
    let rx_source_stock = Regex::new(r"^#([_[:alnum:]]+)").unwrap();
    let rx_source_mod = Regex::new(r"^[0-9]{10}\\[_[:alnum:]]+").unwrap();

    let mut pathbuf = src.to_path_buf();
    let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
    let mut data = Vec::<Category>::with_capacity(subdirs.len());

    for dir_cat in subdirs {
        let dir_name = dir_cat.file_name();
        pathbuf.push(&dir_name);

        let (cat_pfx, cat_name) = get_dir_parts(&dir_name);
        println!("{}: {}", &cat_pfx, &cat_name);

        let mut cat = Category::new(cat_pfx, cat_name);
        let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
        cat.styles.reserve_exact(subdirs.len());

        for dir_style in subdirs.iter() {
            let dir_name = dir_style.file_name();
            pathbuf.push(&dir_name);

            let (style_pfx, style_name) = get_dir_parts(&dir_name);
            println!(" {}: {}", &style_pfx, &style_name);

            let mut style = Style::new(style_pfx, style_name);
            let subdirs: Vec<_> = get_subdirs(&pathbuf).collect();
            style.buildings.reserve_exact(subdirs.len());

            for dir_bld in subdirs {
                let dir_name = dir_bld.file_name();
                pathbuf.push(&dir_name);

                println!("  Building '{}'", dir_name.to_str().unwrap());

                pathbuf.push("building.source");
                buf_sources.clear();
                File::open(&pathbuf).unwrap().read_to_string(&mut buf_sources).unwrap();
                pathbuf.pop(); //pop .source

                let src_type: SourceType = {
                    if let Some(src_stock) = rx_source_stock.captures(&buf_sources) {
                        SourceType::Stock(src_stock.get(1).unwrap().as_str())
                    } else if let Some(src_mod) = rx_source_mod.find(&buf_sources) {
                        SourceType::Mod([ROOT_MODS, src_mod.as_str()].iter().collect())
                    } else {
                        panic!("Cannot parse building source ({:?})", &buf_sources);
                    }
                };

                style.buildings.push(source_to_def(&pathbuf, src_type, stock_buildings));
                pathbuf.pop(); // pop building dir
            }

            cat.styles.push(style);
            pathbuf.pop(); // pop style dir
        }

        data.push(cat);
        pathbuf.pop(); // pop caterory dir
    }

    data
}

#[inline]
fn copy_file(src: &Path, dest: &mut PathBuf, dest_name: &str) {
    dest.push(dest_name);
    fs::copy(src, &dest).unwrap();
    dest.pop();
}


#[inline]
fn copy_file_opt<P>(src: &Option<P>, dest: &mut PathBuf, dest_name: &str) 
where P: AsRef<Path>
{
    if let Some(p) = src.as_ref() {
        copy_file(p.as_ref(), dest, dest_name);
    }
}

fn copy_asset_md5(src: &PathBuf, dest: &PathBuf, buf: &mut Vec<u8>, ext: &str) -> PathBuf {
    use std::convert::TryInto;
    buf.clear();
    
    let mut file = fs::File::open(src).unwrap();
    let meta = file.metadata().unwrap();
    let sz: usize = meta.len().try_into().unwrap();
    if sz > buf.len() {
        buf.reserve(sz);
    }

    file.read_to_end(buf).unwrap();
    let dig = md5::compute(buf.as_mut_slice());
    let filename = format!("{:x}.{}", dig, ext);

    let mut result = dest.clone();
    result.push(filename);
    fs::write(&result, &buf).unwrap();

    result
}


#[inline]
fn copy_asset_md5_opt(src: &Option<PathBuf>, dest: &PathBuf, buf: &mut Vec<u8>, ext: &str) -> Option<PathBuf> {
    src.as_ref().map(|p| copy_asset_md5(p, dest, buf, ext))
}


fn get_dir_parts<'a, 'b>(dir_name: &'a std::ffi::OsStr) -> (&'a str, &'a str) {
    lazy_static! {
        static ref RX: Regex = Regex::new(r"^(\d+?) - (.+)$").unwrap();
    }

    let c = RX.captures(dir_name.to_str().unwrap()).unwrap();
    (c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str())
}


fn get_subdirs(path: &PathBuf) -> impl Iterator<Item=fs::DirEntry>
{
    fs::read_dir(path)
        .unwrap()
        .map(|x| x.unwrap())
        .filter(|x| x.file_type().unwrap().is_dir())
}


#[inline]
fn path_option_valid(opt: &Option<PathBuf>) -> bool {
    match opt {
        None => true,
        Some(ref p) => p.exists()
    }
}


fn parse_ini_to_def<'ini>(render_config: RenderConfig<'ini>) -> BuildingDef<'ini> {
    use const_format::concatcp;
    use path_slash::PathBufExt;

    const SRX_PATH: &str = r"([^\s]+?)(?:\r|\n|\s)";

    lazy_static! {
        static ref RX_MODEL:      Regex = Regex::new(concatcp!(r"\sMODEL\s+?",            SRX_PATH)).unwrap();
        static ref RX_MODEL_LOD1: Regex = Regex::new(concatcp!(r"\sMODEL_LOD\s+?",        SRX_PATH)).unwrap();
        static ref RX_MODEL_LOD2: Regex = Regex::new(concatcp!(r"\sMODEL_LOD2\s+?",       SRX_PATH)).unwrap();
        static ref RX_MODEL_E:    Regex = Regex::new(concatcp!(r"\sMODELEMISSIVE\s+?",    SRX_PATH)).unwrap();

        static ref RX_MATERIAL:   Regex = Regex::new(concatcp!(r"\sMATERIAL\s+?",         SRX_PATH)).unwrap();
        static ref RX_MATERIAL_E: Regex = Regex::new(concatcp!(r"\sMATERIALEMISSIVE\s+?", SRX_PATH)).unwrap();
    }

    let (root, render_source, building_ini, bbox, fire) = match render_config {
        RenderConfig::Stock(key, src) => {
            let bld_ini: PathBuf = [ROOT_STOCK, "buildings_types", &(format!("{}.ini", key))].iter().collect();
            let bbox:    PathBuf = [ROOT_STOCK, "buildings_types", &(format!("{}.bbox", key))].iter().collect();
            let fire:    PathBuf = [ROOT_STOCK, "buildings_types", &(format!("{}.fire", key))].iter().collect();
            (PATH_ROOT_STOCK.as_path(), src, bld_ini, bbox, fire)
        },
        RenderConfig::Mod(_path) => {
            // TODO: read from mod folder
            todo!()
        }
    };

    let grep_path = |rx: &Regex| -> Option<(Range<usize>, PathBuf)> {
        rx.captures(render_source).map(|cap| {
            let m = cap.get(1).unwrap();
            let pth = [root, PathBuf::from_slash(m.as_str()).as_path()] .iter() .collect();
            (m.range(), pth)
        })
    };

    let fire = if fire.exists() { Some(fire) } else { None };

    let p_model = grep_path(&RX_MODEL);
    let p_model_lod1 = grep_path(&RX_MODEL_LOD1);
    let p_model_lod2 = grep_path(&RX_MODEL_LOD2);
    let p_model_emissive = grep_path(&RX_MODEL_E);
    let p_material = grep_path(&RX_MATERIAL);
    let p_material_emissive = grep_path(&RX_MATERIAL_E);

    use std::cmp::Ordering;
    let mut vars = [&p_model, &p_model_lod1, &p_model_lod2, &p_model_emissive, &p_material, &p_material_emissive];
    vars.sort_by(|x, y| {
        match (x, y) {
            (Some((rng1, _)), Some((rng2, _))) => rng1.start.cmp(&rng2.start),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal
        }
    });

    BuildingDef { 
        render_config, building_ini, bbox, fire, 
        model: p_model.unwrap().1, 
        model_lod1: p_model_lod1.map(|x| x.1), 
        model_lod2: p_model_lod2.map(|x| x.1), 
        model_emissive: p_model_emissive.map(|x| x.1), 
        material: p_material.unwrap().1,
        material_emissive: p_material_emissive.map(|x| x.1), 
    }
}




fn get_stock_building<'a, 'ini, 'map>(key: &'a str, hmap: &'map mut StockBuildingsMap<'ini>) -> Option<BuildingDef<'ini>> {
    if let Some(mref) = hmap.get_mut(key) {
        match mref {
            (_, StockBuilding::Parsed(ref x)) => Some(x.clone()),
            (k, StockBuilding::Unparsed(y)) => {
                let x = parse_ini_to_def(RenderConfig::Stock(k, y)); 
                *mref = (k, StockBuilding::Parsed(x.clone()));
                Some(x)
            }
        }
    } else { None }
}


fn source_to_def<'ini, 'map>(dir: &Path, source_type: SourceType, hmap: &'map mut StockBuildingsMap<'ini>) -> BuildingDef<'ini> {
    let mut def = match source_type {
        SourceType::Stock(key) => {
            get_stock_building(&key, hmap).unwrap()
        },
        SourceType::Mod(_path) => {
            todo!()
        }
    };

    let mut pathbuf = dir.to_path_buf();
    
    pathbuf.push("building.ini");
    if pathbuf.exists() { 
        def.building_ini.push(&pathbuf) 
    }

    // TODO: override with other custom files (if they exist in dir)
    //pathbuf.set_file_name(...);

    println!("{}", &def);
    def.validate_paths();
    def
}
