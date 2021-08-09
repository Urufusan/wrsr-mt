mod display;
mod parse;


#[derive(Clone, Copy)]
pub struct Point3f {
    x: f32,
    y: f32,
    z: f32
}


#[derive(Clone, Copy)]
pub struct Rect {
    x1: f32,
    z1: f32,
    x2: f32,
    z2: f32
}


pub enum StrValue<'a> {
    Borrowed(&'a str),
    Owned(String),
}

pub struct QuotedStringParam<'a>(StrValue<'a>);

pub struct IdStringParam<'a>(StrValue<'a>);

pub struct Tagged2Points<T> {
    tag: T,
    p1: Point3f,
    p2: Point3f,
}

//#[derive(Clone)]
pub enum Token<'a> {

    NameStr(QuotedStringParam<'a>),
    Name(u32),

    BuildingType(BuildingType),
    BuildingSubtype(BuildingSubtype),
    HeatEnable,
    HeatDisable,

    CivilBuilding,
    QualityOfLiving(f32),

    WorkersNeeded(u32),
    ProfessorsNeeded(u32),
    CitizenAbleServe(u32),

    Storage((StorageCargoType, f32)),
    StorageFuel((StorageCargoType, f32)),

    RoadNotFlip,
    RoadElectric,
    VehicleStationNotBlock,
    VehicleStationNotBlockDetourPoint(Point3f),
    VehicleStationNotBlockDetourPointPid((u32, Point3f)),
    VehicleStation((Point3f, Point3f)),
    WorkingVehiclesNeeded(u32),

    AirplaneStation(Tagged2Points<AirplaneStationType>),

    Connection2Points(Tagged2Points<Connection2PType>),
    ConnectionRoadDead(Point3f),
    ConnectionAirportDead(Point3f),

    ConnectionsRoadDeadSquare(Rect),
    ConnectionsAirportDeadSquare(Rect),

    Particle((ParticleType, Point3f, f32, f32)),
    TextCaption((Point3f, Point3f)),
    WorkerRenderingArea((Point3f, Point3f)),

    CostWork((ConstructionPhase, f32)),
    CostWorkBuildingNode(IdStringParam<'a>),
    CostWorkBuildingKeyword(IdStringParam<'a>),
    CostWorkBuildingAll,

    CostResource((ResourceType, f32)),
    CostResourceAuto((ConstructionAutoCost, f32)),

    CostWorkVehicleStation((Point3f, Point3f)),
    CostWorkVehicleStationNode(IdStringParam<'a>),
}


impl<'a> Token<'a> {
    const NAME_STR:                       &'static str = "NAME_STR";
    const NAME:                           &'static str = "NAME";
    const BUILDING_TYPE:                  &'static str = "TYPE_";
    const BUILDING_SUBTYPE:               &'static str = "SUBTYPE_";

    const HEATING_ENABLE:                 &'static str = "HEATING_ENABLE";
    const HEATING_DISABLE:                &'static str = "HEATING_DISABLE";
    const CIVIL_BUILDING:                 &'static str = "CIVIL_BUILDING";
    const QUALITY_OF_LIVING:              &'static str = "QUALITY_OF_LIVING";

    const WORKERS_NEEDED:                 &'static str = "WORKERS_NEEDED";
    const PROFESSORS_NEEDED:              &'static str = "PROFESORS_NEEDED";
    const CITIZEN_ABLE_SERVE:             &'static str = "CITIZEN_ABLE_SERVE";

    const STORAGE:                        &'static str = "STORAGE";
    const STORAGE_FUEL:                   &'static str = "STORAGE_FUEL";

    const ROAD_VEHICLE_NOT_FLIP:          &'static str = "ROADVEHICLE_NOTFLIP";
    const ROAD_VEHICLE_ELECTRIC:          &'static str = "ROADVEHICLE_ELETRIC";

    const VEHICLE_STATION_NOT_BLOCK:                  &'static str = "STATION_NOT_BLOCK";
    const VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT:     &'static str = "STATION_NOT_BLOCK_DETOUR_POINT";
    const VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT_PID: &'static str = "STATION_NOT_BLOCK_DETOUR_POINT_PID";

    const VEHICLE_STATION:                &'static str = "VEHICLE_STATION";
    const WORKING_VEHICLES_NEEDED:        &'static str = "WORKING_VEHICLES_NEEDED";

    const AIRPLANE_STATION:               &'static str = "AIRPLANE_STATION_";

    const CONNECTION:                     &'static str = "CONNECTION_";
    const CONNECTION_ROAD_DEAD:           &'static str = "ROAD_DEAD";
    const CONNECTION_AIRPORT_DEAD:        &'static str = "AIRPORT_DEAD";

    const CONNECTIONS_ROAD_DEAD_SQUARE:    &'static str = "CONNECTIONS_ROAD_DEAD_SQUARE";
    const CONNECTIONS_AIRPORT_DEAD_SQUARE: &'static str = "CONNECTIONS_AIRPORT_DEAD_SQUARE";

    const PARTICLE:                       &'static str = "PARTICLE";
    const TEXT_CAPTION:                   &'static str = "TEXT_CAPTION";
    const WORKER_RENDERING_AREA:          &'static str = "WORKER_RENDERING_AREA";


    const COST_WORK:                      &'static str = "COST_WORK";
    const COST_WORK_BUILDING_NODE:        &'static str = "COST_WORK_BUILDING_NODE";
    const COST_WORK_BUILDING_ALL:         &'static str = "COST_WORK_BUILDING_ALL";
    const COST_WORK_BUILDING_KEYWORD:     &'static str = "COST_WORK_BUILDING_KEYWORD";

    const COST_RESOURCE:                  &'static str = "COST_RESOURCE";
    const COST_RESOURCE_AUTO:             &'static str = "COST_RESOURCE_AUTO";
    const COST_WORK_VEHICLE_STATION     : &'static str = "COST_WORK_VEHICLE_STATION";
    const COST_WORK_VEHICLE_STATION_NODE: &'static str = "COST_WORK_VEHICLE_STATION_ACCORDING_NODE";

    pub fn maybe_scale(&self, _factor: f64) -> Option<Self> {
        match self {
            // TODO: process all geometry variants
            //Self::Connection2Points((t, p1, p2)) => Some(Self::Connection2Points((*t, scale_point(factor, *p1), scale_point(factor, *p2)))),
            _ => None
        }
    }
}


impl<'a> super::IniToken<'a> for Token<'a> {
    fn serialize<W: std::io::Write>(&self, wr: W) -> Result<(), std::io::Error>{
        self.serialize_token(wr)
    }

    fn parse_tokens(src: &'a str) -> Vec<(&'a str, super::ParseResult<'a, Self>)> {
        parse::parse_tokens_all(src)
    }

    fn parse_strict(src: &'a str) -> Result<Vec<(&'a str, Self)>, Vec<(&'a str, super::ParseError)>> {
        parse::parse_tokens_strict(src)
    }
}


#[derive(Clone)]
pub enum BuildingType {
    AirplaneGate,
    AirplaneParking,
    AirplaneTower,
    Attraction,
    Broadcast,
    CarDealer,
    CargoStation,
    Church,
    Cityhall,
    ConstructionOffice,
    ConstructionOfficeRail,
    ContainerFacility,
    CoolingTower,
    Customhouse,
    DistributionOffice,
    ElectricExport,
    ElectricImport,
    Engine,
    Factory,
    Farm,
    Field,
    Firestation,
    ForkliftGarage,
    GarbageOffice,
    GasStation,
    HeatingEndstation,
    HeatingPlant,
    HeatingSwitch,
    Hospital,
    Hotel,
    Kindergarten,
    Kino,
    Living,
    MineBauxite,
    MineCoal,
    MineGravel,
    MineIron,
    MineOil,
    MineUranium,
    MineWood,
    Monument,
    Parking,
    PassangerStation,
    PedestrianBridge,
    PoliceStation,
    PollutionMeter,
    Powerplant,
    ProductionLine,
    Pub,
    RailTrafo,
    Raildepo,
    Roaddepo,
    School,
    ShipDock,
    Shop,
    Sport,
    Storage,
    Substation,
    Transformator,
    University,
}


impl BuildingType {
    const TYPE_AIRPLANE_GATE:            &'static str = "AIRPLANE_GATE";
    const TYPE_AIRPLANE_PARKING:         &'static str = "AIRPLANE_PARKING";
    const TYPE_AIRPLANE_TOWER:           &'static str = "AIRPLANE_TOWER";
    const TYPE_ATTRACTION:               &'static str = "ATTRACTION";
    const TYPE_BROADCAST:                &'static str = "BROADCAST";
    const TYPE_CAR_DEALER:               &'static str = "CAR_DEALER";
    const TYPE_CARGO_STATION:            &'static str = "CARGO_STATION";
    const TYPE_CHURCH:                   &'static str = "CHURCH";
    const TYPE_CITYHALL:                 &'static str = "CITYHALL";
    const TYPE_CONSTRUCTION_OFFICE:      &'static str = "CONSTRUCTION_OFFICE";
    const TYPE_CONSTRUCTION_OFFICE_RAIL: &'static str = "CONSTRUCTION_OFFICE_RAIL";
    const TYPE_CONTAINER_FACILITY:       &'static str = "CONTAINER_FACILITY";
    const TYPE_COOLING_TOWER:            &'static str = "COOLING_TOWER";
    const TYPE_CUSTOMHOUSE:              &'static str = "CUSTOMHOUSE";
    const TYPE_DISTRIBUTION_OFFICE:      &'static str = "DISTRIBUTION_OFFICE";
    const TYPE_ELETRIC_EXPORT:           &'static str = "ELETRIC_EXPORT";
    const TYPE_ELETRIC_IMPORT:           &'static str = "ELETRIC_IMPORT";
    const TYPE_ENGINE:                   &'static str = "ENGINE";
    const TYPE_FACTORY:                  &'static str = "FACTORY";
    const TYPE_FARM:                     &'static str = "FARM";
    const TYPE_FIELD:                    &'static str = "FIELD";
    const TYPE_FIRESTATION:              &'static str = "FIRESTATION";
    const TYPE_FORKLIFT_GARAGE:          &'static str = "FORKLIFT_GARAGE"; 
    const TYPE_GARBAGE_OFFICE:           &'static str = "GARBAGE_OFFICE"; 
    const TYPE_GAS_STATION:              &'static str = "GAS_STATION";
    const TYPE_HEATING_ENDSTATION:       &'static str = "HEATING_ENDSTATION";
    const TYPE_HEATING_PLANT:            &'static str = "HEATING_PLANT";
    const TYPE_HEATING_SWITCH:           &'static str = "HEATING_SWITCH";
    const TYPE_HOSPITAL:                 &'static str = "HOSPITAL";
    const TYPE_HOTEL:                    &'static str = "HOTEL";
    const TYPE_KINDERGARTEN:             &'static str = "KINDERGARTEN";
    const TYPE_KINO:                     &'static str = "KINO";
    const TYPE_LIVING:                   &'static str = "LIVING";
    const TYPE_MINE_BAUXITE:             &'static str = "MINE_BAUXITE";
    const TYPE_MINE_COAL:                &'static str = "MINE_COAL";
    const TYPE_MINE_GRAVEL:              &'static str = "MINE_GRAVEL";
    const TYPE_MINE_IRON:                &'static str = "MINE_IRON";
    const TYPE_MINE_OIL:                 &'static str = "MINE_OIL";
    const TYPE_MINE_URANIUM:             &'static str = "MINE_URANIUM";
    const TYPE_MINE_WOOD:                &'static str = "MINE_WOOD";
    const TYPE_MONUMENT:                 &'static str = "MONUMENT";
    const TYPE_PARKING:                  &'static str = "PARKING";
    const TYPE_PASSANGER_STATION:        &'static str = "PASSANGER_STATION";
    const TYPE_PEDESTRIAN_BRIDGE:        &'static str = "PEDESTRIAN_BRIDGE";
    const TYPE_POLICE_STATION:           &'static str = "POLICE_STATION";
    const TYPE_POLLUTION_METER:          &'static str = "POLLUTION_METER";
    const TYPE_POWERPLANT:               &'static str = "POWERPLANT";
    const TYPE_PRODUCTION_LINE:          &'static str = "PRODUCTION_LINE";
    const TYPE_PUB:                      &'static str = "PUB";
    const TYPE_RAIL_TRAFO:               &'static str = "RAIL_TRAFO";
    const TYPE_RAILDEPO:                 &'static str = "RAILDEPO";
    const TYPE_ROADDEPO:                 &'static str = "ROADDEPO";
    const TYPE_SCHOOL:                   &'static str = "SCHOOL";
    const TYPE_SHIP_DOCK:                &'static str = "SHIP_DOCK";
    const TYPE_SHOP:                     &'static str = "SHOP";
    const TYPE_SPORT:                    &'static str = "SPORT";
    const TYPE_STORAGE:                  &'static str = "STORAGE";
    const TYPE_SUBSTATION:               &'static str = "SUBSTATION";
    const TYPE_TRANSFORMATOR:            &'static str = "TRANSFORMATOR";
    const TYPE_UNIVERSITY:               &'static str = "UNIVERSITY";
}


#[derive(Clone)]
pub enum BuildingSubtype {
    Aircustom,
    Airplane,
    Cableway,
    Hostel,
    Medical,
    Radio,
    Rail,
    Restaurant,
    Road,
    Ship,
    Soviet,
    SpaceForVehicles,
    Technical,
    Television,
    Trolleybus,
}


impl BuildingSubtype {
    const SUBTYPE_AIRCUSTOM:          &'static str = "AIRCUSTOM";
    const SUBTYPE_AIRPLANE:           &'static str = "AIRPLANE";
    const SUBTYPE_CABLEWAY:           &'static str = "CABLEWAY";
    const SUBTYPE_HOSTEL:             &'static str = "HOSTEL";
    const SUBTYPE_MEDICAL:            &'static str = "MEDICAL";
    const SUBTYPE_RADIO:              &'static str = "RADIO";
    const SUBTYPE_RAIL:               &'static str = "RAIL";
//  const SUBTYPE_RAL:                &'static str = "RAL";
    const SUBTYPE_RESTAURANT:         &'static str = "RESTAURANT";
    const SUBTYPE_ROAD:               &'static str = "ROAD";
    const SUBTYPE_SHIP:               &'static str = "SHIP";
    const SUBTYPE_SOVIET:             &'static str = "SOVIET";
    const SUBTYPE_SPACE_FOR_VEHICLES: &'static str = "SPACE_FOR_VEHICLES";
    const SUBTYPE_TECHNICAL:          &'static str = "TECHNICAL";
    const SUBTYPE_TELEVISION:         &'static str = "TELEVISION";
    const SUBTYPE_TROLLEYBUS:         &'static str = "TROLLEYBUS";
}


#[derive(Clone)]
pub enum StorageCargoType {
    Passanger,
    Cement,
    Covered,
    Gravel,
    Oil,
    Open,
    Cooler,
    Concrete,
    Livestock,
    General,
}


impl StorageCargoType {
    const PASSANGER: &'static str = "RESOURCE_TRANSPORT_PASSANGER";
    const CEMENT:    &'static str = "RESOURCE_TRANSPORT_CEMENT";
    const COVERED:   &'static str = "RESOURCE_TRANSPORT_COVERED";
    const GRAVEL:    &'static str = "RESOURCE_TRANSPORT_GRAVEL";
    const OIL:       &'static str = "RESOURCE_TRANSPORT_OIL";
    const OPEN:      &'static str = "RESOURCE_TRANSPORT_OPEN";
    const COOLER:    &'static str = "RESOURCE_TRANSPORT_COOLER";
    const CONCRETE:  &'static str = "RESOURCE_TRANSPORT_CONCRETE";
    const LIVESTOCK: &'static str = "RESOURCE_TRANSPORT_LIVESTOCK";
    const GENERAL:   &'static str = "RESOURCE_TRANSPORT_GENERAL";
}


#[derive(Clone)]
pub enum ParticleType {
    ResidentialHeating,
    BigBlack,
    MediumBlack,
    SmallBlack,
    BigGray,
    MediumGray,
    SmallGray,
    BigWhite,
    MediumWhite,
    SmallWhite,
}

impl ParticleType {
    const RESIDENTIAL_HEATING : &'static str = "residential_heating";
    const FACTORY_BIG_BLACK   : &'static str = "factory_big_black";
    const FACTORY_MEDIUM_BLACK: &'static str = "factory_medium_black";
    const FACTORY_SMALL_BLACK : &'static str = "factory_small_black";
    const FACTORY_BIG_GRAY    : &'static str = "factory_big_gray";
    const FACTORY_MEDIUM_GRAY : &'static str = "factory_medium_gray";
    const FACTORY_SMALL_GRAY  : &'static str = "factory_small_gray";
    const FACTORY_BIG_WHITE   : &'static str = "factory_big_white";
    const FACTORY_MEDIUM_WHITE: &'static str = "factory_medium_white";
    const FACTORY_SMALL_WHITE : &'static str = "factory_small_white";
}


#[derive(Clone)]
pub enum ConstructionPhase {
    Groundworks,
    BoardsLaying,
    BricksLaying,
    SkeletonCasting,
    SteelLaying,
    PanelsLaying,
    RooftopBuilding,
    WireLaying,
    Tunneling,
}


impl ConstructionPhase {
    const GROUNDWORKS:      &'static str = "SOVIET_CONSTRUCTION_GROUNDWORKS";
    const BOARDS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BOARDS_LAYING";
    const BRICKS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BRICKS_LAYING";
    const SKELETON_CASTING: &'static str = "SOVIET_CONSTRUCTION_SKELETON_CASTING";
    const STEEL_LAYING:     &'static str = "SOVIET_CONSTRUCTION_STEEL_LAYING";
    const PANELS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_PANELS_LAYING";
    const ROOFTOP_BUILDING: &'static str = "SOVIET_CONSTRUCTION_ROOFTOP_BUILDING";
    const WIRE_LAYING:      &'static str = "SOVIET_CONSTRUCTION_WIRE_LAYING";
    const TUNNELING:        &'static str = "SOVIET_CONSTRUCTION_TUNNELING";
}


#[derive(Clone)]
pub enum ConstructionAutoCost {
    Ground,
    GroundAsphalt,
    WallConcrete,
    WallPanels,
    WallBrick,
    WallSteel,
    WallWood,
    TechSteel,
    ElectroSteel,
    RoofWoodBrick,
    RoofSteel,
    RoofWoodSteel
}


impl ConstructionAutoCost {
    const GROUND:          &'static str = "ground";
    const GROUND_ASPHALT:  &'static str = "ground_asphalt";
    const WALL_CONCRETE:   &'static str = "wall_concrete";
    const WALL_PANELS:     &'static str = "wall_panels";
    const WALL_BRICK:      &'static str = "wall_brick";
    const WALL_STEEL:      &'static str = "wall_steel";
    const WALL_WOOD:       &'static str = "wall_wood";
    const TECH_STEEL:      &'static str = "tech_steel";
    const ELECTRO_STEEL:   &'static str = "electro_steel";
    const ROOF_WOOD_BRICK: &'static str = "roof_woodbrick";
    const ROOF_STEEL:      &'static str = "roof_steel";
    const ROOF_WOOD_STEEL: &'static str = "roof_woodsteel";
}


#[derive(Clone)]
pub enum ResourceType {
    Alcohol,
    Alumina,
    Aluminium,
    Asphalt,
    Bauxite,
    Boards,
    Bricks,
    Chemicals,
    Clothes,
    Concrete,
    ElectroComponents,
    Electricity,
    Electronics,
    Food,
    Gravel,
    MechComponents,
    Meat,
    NuclearFuel,
    Oil,
    Crops,
    PrefabPanels,
    Steel,
    UF6,
    Uranium,
    Wood,
    Workers,
    Yellowcake,
}


impl ResourceType {
    const ALCOHOL:      &'static str = "alcohol";
    const ALUMINA:      &'static str = "alumina";
    const ALUMINIUM:    &'static str = "aluminium";
    const ASPHALT:      &'static str = "asphalt";
    const BAUXITE:      &'static str = "bauxite";
    const BOARDS:       &'static str = "boards";
    const BRICKS:       &'static str = "bricks";
    const CHEMICALS:    &'static str = "chemicals";
    const CLOTHES:      &'static str = "clothes";
    const CONCRETE:     &'static str = "concrete";
    const ELECTRO_COMP: &'static str = "ecomponents";
    const ELECTRICITY:  &'static str = "eletric";
    const ELECTRONICS:  &'static str = "eletronics";
    const FOOD:         &'static str = "food";
    const GRAVEL:       &'static str = "gravel";
    const MECH_COMP:    &'static str = "mcomponents";
    const MEAT:         &'static str = "meat";
    const NUCLEAR_FUEL: &'static str = "nuclearfuel";
    const OIL:          &'static str = "oil";
    const CROPS:        &'static str = "plants";
    const PREFABS:      &'static str = "prefabpanels";
    const STEEL:        &'static str = "steel";
    const UF_6:         &'static str = "uf6";
    const URANIUM:      &'static str = "uranium";
    const WOOD:         &'static str = "wood";
    const WORKERS:      &'static str = "workers";
    const YELLOWCAKE:   &'static str = "yellowcake";
}


#[derive(Clone, Copy)]
pub enum Connection2PType {
    AirRoad,
    Pedestrian,
    Road,
    RoadAllowpass,
    RoadBorder,
    RoadIn,
    RoadOut,
    Rail,
    RailAllowpass,
    RailBorder,
    HeatingBig,
    HeatingSmall,
    SteamIn,
    SteamOut,
    PipeIn,
    PipeOut,
    BulkIn,
    BulkOut,
    Cableway,
    Factory,
    ConveyorIn,
    ConveyorOut,
    ElectricHighIn,
    ElectricHighOut,
    ElectricLowIn,
    ElectricLowOut,
}

impl Connection2PType {
    const CONN_AIRROAD:        &'static str = "AIRROAD";
    const CONN_PEDESTRIAN:     &'static str = "PEDESTRIAN";
    const CONN_ROAD:           &'static str = "ROAD";
    const CONN_ROAD_ALLOWPASS: &'static str = "ROAD_ALLOWPASS";
    const CONN_ROAD_BORDER:    &'static str = "ROAD_BORDER";
    const CONN_ROAD_IN:        &'static str = "ROAD_INPUT";
    const CONN_ROAD_OUT:       &'static str = "ROAD_OUTPUT";
    const CONN_RAIL:           &'static str = "RAIL";
    const CONN_RAIL_ALLOWPASS: &'static str = "RAIL_ALLOWPASS";
    const CONN_RAIL_BORDER:    &'static str = "RAIL_BORDER";
    const CONN_HEATING_BIG:    &'static str = "HEATING_BIG";
    const CONN_HEATING_SMALL:  &'static str = "HEATING_SMALL";
    const CONN_STEAM_IN:       &'static str = "STEAM_INPUT";
    const CONN_STEAM_OUT:      &'static str = "STEAM_OUTPUT";
    const CONN_PIPE_IN:        &'static str = "PIPE_INPUT";
    const CONN_PIPE_OUT:       &'static str = "PIPE_OUTPUT";
    const CONN_BULK_IN:        &'static str = "BULK_INPUT";
    const CONN_BULK_OUT:       &'static str = "BULK_OUTPUT";
    const CONN_CABLEWAY:       &'static str = "CABLEWAY";
    const CONN_FACTORY:        &'static str = "CONNECTION";
    const CONN_CONVEYOR_IN:    &'static str = "CONVEYOR_INPUT";
    const CONN_CONVEYOR_OUT:   &'static str = "CONVEYOR_OUTPUT";
    const CONN_ELECTRIC_H_IN:  &'static str = "ELETRIC_HIGH_INPUT";
    const CONN_ELECTRIC_H_OUT: &'static str = "ELETRIC_HIGH_OUTPUT";
    const CONN_ELECTRIC_L_IN:  &'static str = "ELETRIC_LOW_INPUT";
    const CONN_ELECTRIC_L_OUT: &'static str = "ELETRIC_LOW_OUTPUT";
}


#[derive(Clone, Copy)]
pub enum AirplaneStationType {
    M30,
    M40,
    M50,
    M75,
}

impl AirplaneStationType {
    const AIRPLANE_STATION_30M: &'static str = "30M";
    const AIRPLANE_STATION_40M: &'static str = "40M";
    const AIRPLANE_STATION_50M: &'static str = "50M";
    const AIRPLANE_STATION_75M: &'static str = "75M";
}


#[inline]
fn scale_point(factor: f64, mut p: Point3f) -> Point3f {
    p.x = ((p.x as f64) * factor) as f32;
    p.y = ((p.y as f64) * factor) as f32;
    p.z = ((p.z as f64) * factor) as f32;
    p
}
