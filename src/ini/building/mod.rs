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


#[derive(Clone)]
pub enum StrValue<'a> {
    Borrowed(&'a str),
    Owned(String),
}

#[derive(Clone)]
pub struct QuotedStringParam<'a>(StrValue<'a>);

#[derive(Clone)]
pub struct IdStringParam<'a>(StrValue<'a>);


#[derive(Clone)]
pub enum Token<'a> {

    NameStr(QuotedStringParam<'a>),
    Name(u32),

    BuildingType(BuildingType),
    CivilBuilding,
    QualityOfLiving(f32),

    WorkersNeeded(u32),
    CitizenAbleServe(u32),

    Storage((StorageCargoType, f32)),

    ConnectionPedestrian((Point3f, Point3f)),
    ConnectionRoad((Point3f, Point3f)),
    ConnectionRoadDead(Point3f),
    ConnectionsRoadDeadSquare(Rect),

    Particle((ParticleType, Point3f, f32, f32)),
    TextCaption((Point3f, Point3f)),

    CostWork((ConstructionPhase, f32)),
    CostWorkBuildingNode(IdStringParam<'a>),

    CostResource((ResourceType, f32)),
    CostResourceAuto((ConstructionAutoCost, f32)),

    CostWorkVehicleStation((Point3f, Point3f)),
    CostWorkVehicleStationNode(IdStringParam<'a>),
}


impl<'a> Token<'a> {
    const NAME_STR:                       &'static str = "NAME_STR";
    const NAME:                           &'static str = "NAME";
    const BUILDING_TYPE:                  &'static str = "TYPE_";
    const CIVIL_BUILDING:                 &'static str = "CIVIL_BUILDING";
    const QUALITY_OF_LIVING:              &'static str = "QUALITY_OF_LIVING";
    const WORKERS_NEEDED:                 &'static str = "WORKERS_NEEDED";
    const CITIZEN_ABLE_SERVE:             &'static str = "CITIZEN_ABLE_SERVE";
    const STORAGE:                        &'static str = "STORAGE";
    const CONNECTION_PEDESTRIAN:          &'static str = "CONNECTION_PEDESTRIAN";
    const CONNECTION_ROAD:                &'static str = "CONNECTION_ROAD";
    const CONNECTION_ROAD_DEAD:           &'static str = "CONNECTION_ROAD_DEAD";
    const CONNECTIONS_ROAD_DEAD_SQUARE:   &'static str = "CONNECTIONS_ROAD_DEAD_SQUARE";
    const PARTICLE:                       &'static str = "PARTICLE";
    const TEXT_CAPTION:                   &'static str = "TEXT_CAPTION";
    const COST_WORK:                      &'static str = "COST_WORK";
    const COST_WORK_BUILDING_NODE:        &'static str = "COST_WORK_BUILDING_NODE";
    const COST_RESOURCE:                  &'static str = "COST_RESOURCE";
    const COST_RESOURCE_AUTO:             &'static str = "COST_RESOURCE_AUTO";
    const COST_WORK_VEHICLE_STATION     : &'static str = "COST_WORK_VEHICLE_STATION";
    const COST_WORK_VEHICLE_STATION_NODE: &'static str = "COST_WORK_VEHICLE_STATION_ACCORDING_NODE";

    pub fn maybe_scale(&self, factor: f64) -> Option<Self> {
        match self {
            Self::ConnectionPedestrian(p) => Some(Self::ConnectionPedestrian(scale_2_points(factor, *p))),
            Self::ConnectionRoad(p)       => Some(Self::ConnectionRoad(scale_2_points(factor, *p))),
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
    const AIRPLANE_GATE:            &'static str = "AIRPLANE_GATE";
    const AIRPLANE_PARKING:         &'static str = "AIRPLANE_PARKING";
    const AIRPLANE_TOWER:           &'static str = "AIRPLANE_TOWER";
    const ATTRACTION:               &'static str = "ATTRACTION";
    const BROADCAST:                &'static str = "BROADCAST";
    const CAR_DEALER:               &'static str = "CAR_DEALER";
    const CARGO_STATION:            &'static str = "CARGO_STATION";
    const CHURCH:                   &'static str = "CHURCH";
    const CITYHALL:                 &'static str = "CITYHALL";
    const CONSTRUCTION_OFFICE:      &'static str = "CONSTRUCTION_OFFICE";
    const CONSTRUCTION_OFFICE_RAIL: &'static str = "CONSTRUCTION_OFFICE_RAIL";
    const CONTAINER_FACILITY:       &'static str = "CONTAINER_FACILITY";
    const COOLING_TOWER:            &'static str = "COOLING_TOWER";
    const CUSTOMHOUSE:              &'static str = "CUSTOMHOUSE";
    const DISTRIBUTION_OFFICE:      &'static str = "DISTRIBUTION_OFFICE";
    const ELETRIC_EXPORT:           &'static str = "ELETRIC_EXPORT";
    const ELETRIC_IMPORT:           &'static str = "ELETRIC_IMPORT";
    const ENGINE:                   &'static str = "ENGINE";
    const FACTORY:                  &'static str = "FACTORY";
    const FARM:                     &'static str = "FARM";
    const FIELD:                    &'static str = "FIELD";
    const FIRESTATION:              &'static str = "FIRESTATION";
    const FORKLIFT_GARAGE:          &'static str = "FORKLIFT_GARAGE";
    const GAS_STATION:              &'static str = "GAS_STATION";
    const HEATING_ENDSTATION:       &'static str = "HEATING_ENDSTATION";
    const HEATING_PLANT:            &'static str = "HEATING_PLANT";
    const HEATING_SWITCH:           &'static str = "HEATING_SWITCH";
    const HOSPITAL:                 &'static str = "HOSPITAL";
    const HOTEL:                    &'static str = "HOTEL";
    const KINDERGARTEN:             &'static str = "KINDERGARTEN";
    const KINO:                     &'static str = "KINO";
    const LIVING:                   &'static str = "LIVING";
    const MINE_BAUXITE:             &'static str = "MINE_BAUXITE";
    const MINE_COAL:                &'static str = "MINE_COAL";
    const MINE_GRAVEL:              &'static str = "MINE_GRAVEL";
    const MINE_IRON:                &'static str = "MINE_IRON";
    const MINE_OIL:                 &'static str = "MINE_OIL";
    const MINE_URANIUM:             &'static str = "MINE_URANIUM";
    const MINE_WOOD:                &'static str = "MINE_WOOD";
    const MONUMENT:                 &'static str = "MONUMENT";
    const PARKING:                  &'static str = "PARKING";
    const PASSANGER_STATION:        &'static str = "PASSANGER_STATION";
    const POLLUTION_METER:          &'static str = "POLLUTION_METER";
    const POWERPLANT:               &'static str = "POWERPLANT";
    const PRODUCTION_LINE:          &'static str = "PRODUCTION_LINE";
    const PUB:                      &'static str = "PUB";
    const RAIL_TRAFO:               &'static str = "RAIL_TRAFO";
    const RAILDEPO:                 &'static str = "RAILDEPO";
    const ROADDEPO:                 &'static str = "ROADDEPO";
    const SCHOOL:                   &'static str = "SCHOOL";
    const SHIP_DOCK:                &'static str = "SHIP_DOCK";
    const SHOP:                     &'static str = "SHOP";
    const SPORT:                    &'static str = "SPORT";
    const STORAGE:                  &'static str = "STORAGE";
    const SUBSTATION:               &'static str = "SUBSTATION";
    const TRANSFORMATOR:            &'static str = "TRANSFORMATOR";
    const UNIVERSITY:               &'static str = "UNIVERSITY";
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


#[inline]
fn scale_point(factor: f64, mut p: Point3f) -> Point3f {
    p.x = ((p.x as f64) * factor) as f32;
    p.y = ((p.y as f64) * factor) as f32;
    p.z = ((p.z as f64) * factor) as f32;
    p
}

#[inline]
fn scale_2_points(factor: f64, (p1, p2): (Point3f, Point3f)) -> (Point3f, Point3f) {
    (scale_point(factor, p1), scale_point(factor, p2))
}
